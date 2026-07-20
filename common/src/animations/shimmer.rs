use crate::animations::{LEDS_TOTAL, LedData};
use rand::rngs::SmallRng;
use rand::{RngExt, SeedableRng};
use smart_leds::hsv::{Hsv, hsv2rgb};

pub struct Shimmer {
    hsv: Hsv,
}

impl Shimmer {
    pub(crate) const BRIGHTNESS_DAMPING_FACTOR: f32 = 0.2;

    pub(crate) fn new(random_seed: u64) -> Self {
        let mut prng = SmallRng::seed_from_u64(random_seed);
        Self {
            hsv: Hsv {
                // Start the animation with a random hue.
                hue: prng.random_range(0..=u8::MAX),
                sat: 255,
                val: 255,
            },
        }
    }

    pub(crate) fn update(&mut self, data: &mut LedData) {
        let rgb = hsv2rgb(self.hsv);
        *data = [rgb; LEDS_TOTAL];
        self.hsv.hue = self.hsv.hue.wrapping_add(1);
    }
}
