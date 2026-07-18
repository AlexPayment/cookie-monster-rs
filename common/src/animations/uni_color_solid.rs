use crate::animations::{COLORS, LEDS_TOTAL, LedData, Settings};

const BRIGHTNESS_DAMPING_FACTOR: f32 = 0.05;

pub struct UniColorSolid {}

impl UniColorSolid {
    pub(crate) fn new() -> Self {
        Self {}
    }

    pub(crate) fn brightness_damping_factor() -> f32 {
        BRIGHTNESS_DAMPING_FACTOR
    }

    pub(crate) fn update(&mut self, data: &mut LedData, settings: &Settings) {
        *data = [COLORS[settings.color_index()]; LEDS_TOTAL];
    }
}
