use cookie_monster_common::signal::{
    ANIMATION_CHANGED_SIGNAL, BRIGHTNESS_READ_SIGNAL, COLOR_CHANGED_SIGNAL, DELAY_READ_SIGNAL,
};
use defmt::{debug, info};
use embassy_nrf::gpio::{AnyPin, Input, Pull};
use embassy_nrf::peripherals::SAADC;
use embassy_nrf::saadc::{AnyInput, ChannelConfig, Config, Saadc};
use embassy_nrf::{bind_interrupts, saadc};
use embassy_time::Delay;
use embedded_hal_async::delay::DelayNs;

bind_interrupts!(struct Irqs {
    SAADC => saadc::InterruptHandler;
});

const ANALOG_SENSORS_READ_FREQUENCY_MILLISECONDS: u32 = 500;
const DEBOUNCE_PERIOD_MILLISECONDS: u32 = 50;

/// Task that reads analog sensors (potentiometers) to signal the brightness and delay values.
///
/// All sensors are connected to SAADC, which has a 12-bit resolution. Unfortunately, embassy
/// doesn't allow a task to be generic.
#[embassy_executor::task]
pub async fn analog_sensors_task(adc: SAADC, brightness_pin: AnyInput, delay_pin: AnyInput) {
    info!("Starting analog sensors task...");

    let mut saadc = configure_adc(adc, brightness_pin, delay_pin).await;

    let mut buffer = [0; 2];
    let mut delay = Delay;

    loop {
        // Read the brightness and delay values from the potentiometers.
        saadc.sample(&mut buffer).await;

        let brightness_value = u16::try_from(buffer[0]).unwrap_or_default();
        let delay_value = u16::try_from(buffer[1]).unwrap_or_default();

        BRIGHTNESS_READ_SIGNAL.signal(brightness_value);
        DELAY_READ_SIGNAL.signal(delay_value);

        info!("Brightness: {}, Delay: {}", brightness_value, delay_value);

        // Wait for a short period before reading again.
        delay
            .delay_ms(ANALOG_SENSORS_READ_FREQUENCY_MILLISECONDS)
            .await;
    }
}

/// Task that waits for a button to be pressed to signal an animation change.
#[embassy_executor::task]
pub async fn animation_button_task(button: AnyPin) {
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
pub async fn color_button_task(button: AnyPin) {
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
    adc: SAADC, brightness_pin: AnyInput, delay_pin: AnyInput,
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
