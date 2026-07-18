use crate::animations::{COLORS, COLORS_TOTAL, LEDS_TOTAL, LedData};

const BRIGHTNESS_DAMPING_FACTOR: f32 = 0.05;
const LEDS_PER_COLOR: usize = LEDS_TOTAL / COLORS_TOTAL;

pub struct MultiColorSolid {}

impl MultiColorSolid {
    pub(crate) fn new() -> Self {
        Self {}
    }

    pub(crate) fn brightness_damping_factor() -> f32 {
        BRIGHTNESS_DAMPING_FACTOR
    }

    pub(crate) fn update(&mut self, data: &mut LedData) {
        let mut color_index = 0;

        for (i, led) in data.iter_mut().enumerate() {
            if i % LEDS_PER_COLOR == 0 {
                color_index += 1;
            }
            *led = COLORS[color_index % COLORS_TOTAL];
        }
    }
}
