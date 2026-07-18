use crate::animations::{COLORS, COLORS_TOTAL, LEDS_TOTAL, LedData};
use rand::rngs::SmallRng;
use rand::{RngExt, SeedableRng};

const BRIGHTNESS_DAMPING_FACTOR: f32 = 0.05;

pub struct Carrousel {
    color_index: usize,
    position: usize,
    prng: SmallRng,
}

impl Carrousel {
    pub(crate) fn new(random_seed: u64) -> Self {
        let mut prng = SmallRng::seed_from_u64(random_seed);
        let color_index = prng.random_range(0..COLORS_TOTAL);
        Self {
            color_index,
            position: 0,
            prng,
        }
    }

    pub(crate) fn brightness_damping_factor() -> f32 {
        BRIGHTNESS_DAMPING_FACTOR
    }

    pub(crate) fn update(&mut self, data: &mut LedData) {
        data[self.position] = COLORS[self.color_index];

        self.position += 1;
        if self.position >= LEDS_TOTAL {
            self.position = 0;
            let mut new_color = self.prng.random_range(0..COLORS_TOTAL);
            while self.color_index == new_color {
                // Make sure the new color is different from the current color
                new_color = self.prng.random_range(0..COLORS_TOTAL);
            }
            self.color_index = new_color;
        }
    }
}
