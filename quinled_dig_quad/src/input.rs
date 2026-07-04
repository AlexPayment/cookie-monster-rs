use cookie_monster_common::input::process_analog_sensors;
use cookie_monster_common::signal::{ANIMATION_CHANGED_SIGNAL, COLOR_CHANGED_SIGNAL};
use defmt::{debug, info};
use embassy_time::Delay;
use embedded_hal_async::delay::DelayNs;
use esp_hal::analog::adc::{Adc, AdcConfig, Attenuation};
use esp_hal::gpio::Pull::Up;
use esp_hal::gpio::{AnyPin, Input, InputConfig};
use esp_hal::peripherals::{ADC2, GPIO12, GPIO15};
use nb::block;

const ADC_MAXIMUM_VALUE: u16 = 2u16.pow(ADC_RESOLUTION) - 1;
const ADC_RESOLUTION: u32 = 12;
// This represents 0.5% of the maximum ADC value.
const ANALOG_SENSORS_JITTER_THRESHOLD: u16 = ADC_MAXIMUM_VALUE / 200;
const ANALOG_SENSORS_READ_FREQUENCY_MILLISECONDS: u32 = 500;
const DEBOUNCE_PERIOD_MILLISECONDS: u32 = 50;

pub type BrightnessPin<'a> = GPIO15<'a>;
pub type DelayPin<'a> = GPIO12<'a>;

/// Task that reads analog sensors (potentiometers) to signal the brightness and delay values.
///
/// All sensors are connected to ADC2, which is a 12-bit ADC. Unfortunately, embassy doesn't allow a
/// task to be generic.
#[embassy_executor::task]
pub async fn analog_sensors_task(
    adc: ADC2<'static>, brightness_pin: BrightnessPin<'static>, delay_pin: DelayPin<'static>,
) {
    info!("Starting analog sensors task...");

    // The ESP32 ADC has a resolution of 12 bits, which means the maximum value is 4095.
    let mut adc_config = AdcConfig::default();
    // Because the brightness potentiometer is connected to the 3.3 V pin, we need to set the
    // attenuation to 11 dB to cover the 0 to 3.3 V range.
    let mut brightness_pin = adc_config.enable_pin(brightness_pin, Attenuation::_11dB);
    // Because the delay potentiometer is connected to the 3.3 V pin, we need to set the
    // attenuation to 11 dB to cover the 0 to 3.3 V range.
    let mut delay_pin = adc_config.enable_pin(delay_pin, Attenuation::_11dB);
    let mut adc = Adc::new(adc, adc_config);

    let mut delay = Delay;

    let mut last_brightness: u16 = 0;
    let mut last_delay: u16 = 0;

    loop {
        let (updated_brightness, updated_delay) = process_analog_sensors(
            block!(adc.read_oneshot(&mut brightness_pin)),
            block!(adc.read_oneshot(&mut delay_pin)),
            last_brightness,
            last_delay,
            ANALOG_SENSORS_JITTER_THRESHOLD,
        );

        if let Some(updated_brightness) = updated_brightness {
            last_brightness = updated_brightness;
        }
        if let Some(updated_delay) = updated_delay {
            last_delay = updated_delay;
        }

        // Wait for a short period before reading again.
        delay
            .delay_ms(ANALOG_SENSORS_READ_FREQUENCY_MILLISECONDS)
            .await;
    }
}

/// Task that waits for a button to be pressed to signal an animation change.
#[embassy_executor::task]
pub async fn animation_button_task(button: AnyPin<'static>) {
    info!("Starting animation button task...");

    let mut button = Input::new(button, InputConfig::default().with_pull(Up));

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
pub async fn color_button_task(button: AnyPin<'static>) {
    info!("Starting color button task...");

    let mut button = Input::new(button, InputConfig::default().with_pull(Up));

    loop {
        perform_when_button_pressed(&mut button, || async {
            COLOR_CHANGED_SIGNAL.signal(());
            info!("Color change signaled");
        })
        .await;
    }
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
