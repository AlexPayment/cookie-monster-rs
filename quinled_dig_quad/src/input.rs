use cookie_monster_common::signal::{
    ANIMATION_CHANGED_SIGNAL, BRIGHTNESS_READ_SIGNAL, COLOR_CHANGED_SIGNAL, DELAY_READ_SIGNAL,
};
use defmt::{debug, info};
use embassy_time::Delay;
use embedded_hal_async::delay::DelayNs;
use esp_hal::analog::adc::{Adc, AdcConfig, Attenuation};
use esp_hal::gpio::Pull::Up;
use esp_hal::gpio::{AnyPin, Input, InputConfig};
use esp_hal::peripherals::{ADC2, GPIO12, GPIO15};
use nb::block;

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

    loop {
        // Read the brightness and delay values from the potentiometers.
        let brightness_value: u16 = block!(adc.read_oneshot(&mut brightness_pin)).unwrap();
        let delay_value: u16 = block!(adc.read_oneshot(&mut delay_pin)).unwrap();

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
