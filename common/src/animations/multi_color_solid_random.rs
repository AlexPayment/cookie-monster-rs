use crate::animations::{LEDS_TOTAL, LedData};
use rand::rngs::SmallRng;
use rand::{RngExt, SeedableRng};
use smart_leds::RGB8;

pub struct MultiColorSolidRandom {
    prng: SmallRng,
    rendered_data: [RGB8; LEDS_TOTAL],
}

impl MultiColorSolidRandom {
    pub(crate) const BRIGHTNESS_DAMPING_FACTOR: f32 = 0.2;

    pub(crate) fn new(random_seed: u64) -> Self {
        let mut animation = Self {
            prng: SmallRng::seed_from_u64(random_seed),
            rendered_data: [RGB8::default(); LEDS_TOTAL],
        };

        for i in 0..LEDS_TOTAL {
            let random_color = RGB8::new(
                animation.prng.random_range(0..=u8::MAX),
                animation.prng.random_range(0..=u8::MAX),
                animation.prng.random_range(0..=u8::MAX),
            );
            animation.rendered_data[i] = random_color;
        }

        animation
    }

    pub(crate) fn update(&mut self, data: &mut LedData) {
        data.copy_from_slice(&self.rendered_data);
    }
}
