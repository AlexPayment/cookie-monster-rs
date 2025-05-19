use defmt::Format;

/// Common settings for the animations.
#[derive(Clone, Copy, Debug, Format)]
pub struct Settings {
    brightness: f32,
    color_index: usize,
    delay: u32,
}

impl Settings {
    #[must_use]
    pub fn new(color_index: usize, brightness: f32, delay: u32) -> Self {
        Self {
            brightness,
            color_index,
            delay,
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

    pub fn set_brightness(&mut self, brightness: f32) {
        self.brightness = brightness;
    }

    pub fn set_color_index(&mut self, color_index: usize) {
        self.color_index = color_index;
    }

    pub fn set_delay(&mut self, delay: u32) {
        self.delay = delay;
    }
}
