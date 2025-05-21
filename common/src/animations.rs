use core::cmp;
use defmt::Format;

pub const DEFAULT_COLOR_INDEX: usize = 9;

/// Common settings for the animations.
#[derive(Clone, Copy, Debug, Format)]
pub struct Settings {
    /// Brightness of the LEDs, between 0.0 and 1.0.
    brightness: f32,

    /// Index of the color to be used in the animation.
    ///
    /// Multicolor animations will generally ignore this value.
    color_index: usize,

    /// Delay between frames in milliseconds.
    delay: u32,

    /// Maximum value of the analog sensors (potentiometers).
    max_analog_value: u16,

    /// Number of colors available for the animations.
    num_colors: usize,
}

impl Settings {
    #[must_use]
    pub fn new(
        color_index: usize, brightness: u16, delay: u16, max_analog_value: u16, num_colors: usize,
    ) -> Self {
        Self {
            brightness: calculate_brightness(brightness, max_analog_value),
            color_index,
            delay: calculate_delay(delay, max_analog_value),
            max_analog_value,
            num_colors,
        }
    }

    #[must_use]
    pub fn brightness(&self) -> f32 {
        self.brightness
    }

    #[must_use]
    pub fn color_index(&self) -> usize {
        self.color_index
    }

    #[must_use]
    pub fn delay(&self) -> u32 {
        self.delay
    }

    /// Increment the color index and wrap around if it exceeds the number of colors.
    pub fn increment_color_index(&mut self) {
        self.color_index = (self.color_index + 1) % self.num_colors;
    }

    pub fn set_brightness(&mut self, brightness: u16) {
        self.brightness = calculate_brightness(brightness, self.max_analog_value);
    }

    pub fn set_color_index(&mut self, color_index: usize) {
        self.color_index = color_index;
    }

    pub fn set_delay(&mut self, delay: u16) {
        self.delay = calculate_delay(delay, self.max_analog_value);
    }
}

pub fn calculate_index(value: u16, max_value: u16, num_values: usize) -> usize {
    let index = (f32::from(value) / f32::from(max_value) * num_values as f32) as usize;
    cmp::min(index, num_values - 1)
}

/// Calculate the brightness based on the value of the potentiometer reading.
///
/// The value is between 0 and 1.
fn calculate_brightness(value: u16, max_value: u16) -> f32 {
    f32::from(value) / f32::from(max_value)
}

/// Calculate the delay in milliseconds based on the value of the potentiometer reading.
///
/// The delay is calculated as a fraction of the maximum analog value times one thousand. The
/// resulting value is then clamped to a minimum of 1.
fn calculate_delay(value: u16, max_value: u16) -> u32 {
    cmp::max((f32::from(value) / f32::from(max_value) * 1000.0) as u32, 1)
}
