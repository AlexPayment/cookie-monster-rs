use crate::signal::{BRIGHTNESS_READ_SIGNAL, DELAY_READ_SIGNAL};
use defmt::info;

/// Process analog sensor readings and update signals if necessary.
pub fn process_analog_sensors(
    brightness_reading: Result<u16, ()>, delay_reading: Result<u16, ()>, last_brightness: u16,
    last_delay: u16, jitter_threshold: u16,
) -> (Option<u16>, Option<u16>) {
    let mut updated_brightness = None;
    let mut updated_delay = None;

    if let Ok(raw_brightness) = brightness_reading
        && raw_brightness.abs_diff(last_brightness) > jitter_threshold
    {
        updated_brightness = Some(raw_brightness);
        BRIGHTNESS_READ_SIGNAL.signal(raw_brightness);
    }

    if let Ok(raw_delay) = delay_reading
        && raw_delay.abs_diff(last_delay) > jitter_threshold
    {
        updated_delay = Some(raw_delay);
        DELAY_READ_SIGNAL.signal(raw_delay);
    }

    if updated_brightness.is_some() || updated_delay.is_some() {
        info!("Brightness: {}, Delay: {}", last_brightness, last_delay);
    }

    (updated_brightness, updated_delay)
}
