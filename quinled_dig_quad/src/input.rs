use crate::{AnimationChangedSignal, BrightnessReadSignal, ColorChangedSignal, DelayReadSignal};
use defmt::{debug, info};
use embassy_time::{Duration, Timer};
use esp_hal::analog::adc::{Adc, AdcConfig, Attenuation};
use esp_hal::gpio::Pull::Up;
use esp_hal::gpio::{AnyPin, GpioPin, Input, InputConfig};
use esp_hal::peripherals::ADC2;
use nb::block;

const ANALOG_SENSORS_READ_FREQUENCY: Duration = Duration::from_millis(100);
const DEBOUNCE_PERIOD: Duration = Duration::from_millis(50);

pub type BrightnessPin = GpioPin<15>;
pub type DelayPin = GpioPin<12>;

/// Task that waits for a button to be pressed to signal an animation change.
#[embassy_executor::task]
pub async fn animation_button_task(
    button: AnyPin, animation_changed_signal: &'static AnimationChangedSignal,
) {
    let mut button = Input::new(button, InputConfig::default().with_pull(Up));

    loop {
        perform_when_button_pressed(&mut button, || async {
            animation_changed_signal.signal(());
            info!("Animation change signaled");
        })
        .await;
    }
}

/// Task that reads analog sensors (potentiometers) to signal the brightness and delay values.
///
/// All sensors are connected to ADC2, which is a 12-bit ADC. Unfortunately, embassy doesn't allow a
/// task to be generic.
#[embassy_executor::task]
pub async fn analog_sensors_task(
    adc: ADC2, brightness_pin: BrightnessPin, delay_pin: DelayPin,
    brightness_read_signal: &'static BrightnessReadSignal,
    delay_read_signal: &'static DelayReadSignal,
) {
    // The ESP32 ADC has a resolution of 12 bits, which means the maximum value is 4095.
    let mut adc2_config = AdcConfig::default();
    // Because the brightness potentiometer is connected to the 3.3 V pin, we need to set the
    // attenuation to 11 dB to cover the 0 to 3.3 V range.
    let mut brightness_pin = adc2_config.enable_pin(brightness_pin, Attenuation::_11dB);
    // Because the delay potentiometer is connected to the 3.3 V pin, we need to set the
    // attenuation to 11 dB to cover the 0 to 3.3 V range.
    let mut delay_pin = adc2_config.enable_pin(delay_pin, Attenuation::_11dB);
    let mut adc2 = Adc::new(adc, adc2_config);

    loop {
        // Read the brightness and delay values from the potentiometers.
        let brightness_value: u16 = block!(adc2.read_oneshot(&mut brightness_pin)).unwrap();
        let delay_value: u16 = block!(adc2.read_oneshot(&mut delay_pin)).unwrap();

        brightness_read_signal.signal(brightness_value);
        delay_read_signal.signal(delay_value);

        info!("Brightness: {}, Delay: {}", brightness_value, delay_value);

        // Wait for a short period before reading again.
        Timer::after(ANALOG_SENSORS_READ_FREQUENCY).await;
    }
}

/// Task that waits for a button to be pressed to signal a color change.
#[embassy_executor::task]
pub async fn color_button_task(button: AnyPin, color_change_signal: &'static ColorChangedSignal) {
    let mut button = Input::new(button, InputConfig::default().with_pull(Up));

    loop {
        perform_when_button_pressed(&mut button, || async {
            color_change_signal.signal(());
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
    // Wait for the button to be pressed (falling edge).
    // This will block until the button is pressed.
    button.wait_for_falling_edge().await;

    // Wait for a short debounce period. This allows the physical bouncing to settle. Adjust the
    // duration (e.g., 20 ms, 50 ms, 100 ms) based on the button's characteristics.
    Timer::after(DEBOUNCE_PERIOD).await;

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
        Timer::after(DEBOUNCE_PERIOD).await;
    }

    // If button_pin.is_high() here, it means it was a very short bounce that didn't settle, so we
    // simply loop and wait for the next falling edge.
}
