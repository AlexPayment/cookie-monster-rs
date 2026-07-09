use cookie_monster_common::input::process_analog_sensors;
use cookie_monster_common::signal::{ANIMATION_CHANGED_SIGNAL, COLOR_CHANGED_SIGNAL};
use defmt::{debug, error, info};
use embassy_nrf::gpio::{AnyPin, Input, Pull};
use embassy_nrf::peripherals::SAADC;
use embassy_nrf::saadc::{AnyInput, ChannelConfig, Config, Saadc};
use embassy_nrf::{Peri, bind_interrupts, saadc};
use embassy_time::Delay;
use embedded_hal_async::delay::DelayNs;

bind_interrupts!(struct Irqs {
    SAADC => saadc::InterruptHandler;
});

// The default analog value is set to half of the maximum value, which is 2048.
pub(crate) const ANALOG_DEFAULT_VALUE: u16 = ANALOG_MAXIMUM_VALUE / 2;
// The ADC resolution is 12 bits, which means the maximum value is 4095 (2^12 - 1).
pub(crate) const ANALOG_MAXIMUM_VALUE: u16 = 2u16.pow(ADC_RESOLUTION) - 1;
const ADC_RESOLUTION: u32 = 12;
// This represents 0.5% of the maximum ADC value.
const ANALOG_JITTER_THRESHOLD: u16 = ANALOG_MAXIMUM_VALUE / 200;
const ANALOG_READ_FREQUENCY_MILLISECONDS: u32 = 500;
const DEBOUNCE_PERIOD_MILLISECONDS: u32 = 50;

/// Task that reads analog sensors (potentiometers) to signal the brightness and delay values.
///
/// All sensors are connected to SAADC, which has a 12-bit resolution. Unfortunately, embassy
/// doesn't allow a task to be generic.
#[embassy_executor::task]
pub async fn analog_sensors_task(
    adc: Peri<'static, SAADC>, brightness_pin: AnyInput<'static>, delay_pin: AnyInput<'static>,
) {
    info!("Starting analog sensors task...");

    let mut saadc = configure_adc(adc, brightness_pin, delay_pin).await;

    let mut buffer = [0; 2];
    let mut delay = Delay;

    let mut last_brightness = 0;
    let mut last_delay = 0;

    loop {
        // Read the brightness and delay values from the potentiometers.
        saadc.sample(&mut buffer).await;

        let (updated_brightness, updated_delay) = process_analog_sensors(
            u16::try_from(buffer[0]).map_err(|e| {
                error!(
                    "Cannot convert the value read from the brightness potentiometer: {}",
                    e
                );
            }),
            u16::try_from(buffer[1]).map_err(|e| {
                error!(
                    "Cannot convert the value read from the delay potentiometer: {}",
                    e
                );
            }),
            last_brightness,
            last_delay,
            ANALOG_JITTER_THRESHOLD,
        );

        if let Some(updated_brightness) = updated_brightness {
            last_brightness = updated_brightness;
        }
        if let Some(updated_delay) = updated_delay {
            last_delay = updated_delay;
        }

        // Wait for a short period before reading again.
        delay.delay_ms(ANALOG_READ_FREQUENCY_MILLISECONDS).await;
    }
}

/// Task that waits for a button to be pressed to signal an animation change.
#[embassy_executor::task]
pub async fn animation_button_task(button: Peri<'static, AnyPin>) {
    info!("Starting animation button task...");

    let mut button = Input::new(button, Pull::Up);

    loop {
        perform_when_button_pressed(&mut button, || async {
            ANIMATION_CHANGED_SIGNAL.signal(());
            info!("Animation change signaled");
        })
        .await;
    }
}

/// Task that waits for a button to be pressed to signal a color change.
#[embassy_executor::task]
pub async fn color_button_task(button: Peri<'static, AnyPin>) {
    info!("Starting color button task...");

    let mut button = Input::new(button, Pull::Up);

    loop {
        perform_when_button_pressed(&mut button, || async {
            COLOR_CHANGED_SIGNAL.signal(());
            info!("Color change signaled");
        })
        .await;
    }
}

async fn configure_adc<'a>(
    adc: Peri<'a, SAADC>, brightness_pin: AnyInput<'static>, delay_pin: AnyInput<'static>,
) -> Saadc<'a, 2> {
    info!("Configuring SAADC...");

    // The ADC resolution is 12 bits, which means the maximum value is 4095.
    let config = Config::default();
    let brightness_channel_config = ChannelConfig::single_ended(brightness_pin);
    let delay_channel_config = ChannelConfig::single_ended(delay_pin);

    let saadc = Saadc::new(
        adc,
        Irqs,
        config,
        [brightness_channel_config, delay_channel_config],
    );

    info!("Calibrating SAADC...");
    saadc.calibrate().await;

    saadc
}

/// Executes the provided action when the button is pressed.
///
/// It handles debouncing to ensure that the action is only executed once per press.
async fn perform_when_button_pressed<F, Fut>(button: &mut Input<'_>, action: F)
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = ()>,
{
    debug!("Waiting for button press...");

    let mut delay = Delay;

    // Wait for the button to be pressed (falling edge).
    // This will block until the button is pressed.
    button.wait_for_falling_edge().await;

    // Wait for a short debounce period. This allows the physical bouncing to settle. Adjust the
    // duration (e.g., 20 ms, 50 ms, 100 ms) based on the button's characteristics.
    delay.delay_ms(DEBOUNCE_PERIOD_MILLISECONDS).await;

    // After the debounce time, check the *actual* state of the pin. If it's still low, it's a
    // valid press.
    if button.is_low() {
        // Valid button press detected.
        debug!("Button pressed!");
        // Perform the action.
        action().await;

        // Now, wait for the button to be released to prevent multiple triggers if the button is
        // held down, and also to allow for the next press.
        button.wait_for_rising_edge().await;

        // Add a small delay after release to debounce the release too.
        delay.delay_ms(DEBOUNCE_PERIOD_MILLISECONDS).await;
    }

    // If button_pin.is_high() here, it means it was a very short bounce that didn't settle, so we
    // simply loop and wait for the next falling edge.
}
