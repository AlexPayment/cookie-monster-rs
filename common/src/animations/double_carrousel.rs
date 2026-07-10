use crate::animations::{COLORS, LedData, NUM_COLORS, NUM_LEDS, Settings};
use core::fmt::Debug;
use embedded_hal_async::delay::DelayNs;
use rand::rngs::SmallRng;
use rand::{RngExt, SeedableRng};
use smart_leds::{RGB8, brightness, gamma};
use smart_leds_trait::SmartLedsWrite;

const LEDS_PER_CARROUSEL: usize = NUM_LEDS / 2;

pub struct DoubleCarrousel {
    color_index_1: usize,
    color_index_2: usize,
    position_1: usize,
    position_2: usize,
    prng: SmallRng,
}

impl DoubleCarrousel {
    pub(crate) fn new(random_seed: u64) -> Self {
        let mut prng = SmallRng::seed_from_u64(random_seed);
        let color_index_1 = prng.random_range(0..NUM_COLORS);
        let color_index_2 = prng.random_range(0..NUM_COLORS);
        Self {
            color_index_1,
            color_index_2,
            position_1: 0,
            position_2: NUM_LEDS - 1,
            prng,
        }
    }

    pub(crate) async fn render(
        &mut self, data: &LedData,
        ws2812: &mut impl SmartLedsWrite<Color = RGB8, Error = impl Debug>,
        delay: &mut impl DelayNs, settings: &Settings,
    ) {
        ws2812
            .write(brightness(
                gamma(data.iter().copied()),
                self.brightness(settings),
            ))
            .unwrap();

        delay.delay_ms(settings.delay()).await;
    }

    pub(crate) fn update(&mut self, data: &mut LedData) {
        data[self.position_1] = COLORS[self.color_index_1];
        data[self.position_2] = COLORS[self.color_index_2];

        self.position_1 += 1;
        if self.position_1 >= LEDS_PER_CARROUSEL {
            self.position_1 = 0;
            let mut new_color = self.prng.random_range(0..NUM_COLORS);
            while self.color_index_1 == new_color {
                // Make sure the new color is different from the current color
                new_color = self.prng.random_range(0..NUM_COLORS);
            }
            self.color_index_1 = new_color;
        }

        self.position_2 -= 1;
        if self.position_2 <= LEDS_PER_CARROUSEL {
            self.position_2 = NUM_LEDS - 1;
            let mut new_color = self.prng.random_range(0..NUM_COLORS);
            while self.color_index_2 == new_color {
                // Make sure the new color is different from the current color
                new_color = self.prng.random_range(0..NUM_COLORS);
            }
            self.color_index_2 = new_color;
        }
    }

    fn brightness(&self, settings: &Settings) -> u8 {
        (f32::from(settings.brightness()) * 0.05) as u8
    }
}
