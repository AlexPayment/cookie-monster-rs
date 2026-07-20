use crate::animations::{COLORS, LEDS_TOTAL, LedData, Settings};

pub struct UniColorSolid {}

impl UniColorSolid {
    pub(crate) const BRIGHTNESS_DAMPING_FACTOR: f32 = 0.05;

    pub(crate) fn new() -> Self {
        Self {}
    }

    pub(crate) fn update(&mut self, data: &mut LedData, settings: &Settings) {
        *data = [COLORS[settings.color_index()]; LEDS_TOTAL];
    }
}
