use crate::signal::{BRIGHTNESS_READ_SIGNAL, DELAY_READ_SIGNAL};
use defmt::info;

/// Process analog sensor readings and update signals if necessary.
pub fn process_analog_sensors(brightness_reading: Result<u16, ()>, delay_reading: Result<u16, ()>) {
    let mut brightness = None;
    let mut delay = None;

    if let Ok(raw_brightness) = brightness_reading {
        brightness = Some(raw_brightness);
        BRIGHTNESS_READ_SIGNAL.signal(raw_brightness);
    }

    if let Ok(raw_delay) = delay_reading {
        delay = Some(raw_delay);
        DELAY_READ_SIGNAL.signal(raw_delay);
    }

    if brightness.is_some() || delay.is_some() {
        info!("Brightness: {}, Delay: {}", brightness, delay);
    }
}
