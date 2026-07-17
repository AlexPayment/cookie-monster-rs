use crate::animations::{
    COLORS, COLORS_TOTAL, LEDS_SECTION_1_RANGE, LEDS_SECTION_2_RANGE, LEDS_TOTAL, LedData, Settings,
};
use core::fmt::Debug;
use embassy_futures::join::join;
use embedded_hal_async::delay::DelayNs;
use rand::rngs::SmallRng;
use rand::{RngExt, SeedableRng};
use smart_leds::{RGB8, brightness, gamma};
use smart_leds_trait::SmartLedsWrite;

const BRIGHTNESS_DAMPING_FACTOR: f32 = 0.05;
const LEDS_PER_CARROUSEL: usize = LEDS_TOTAL / 2;

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
        let color_index_1 = prng.random_range(0..COLORS_TOTAL);
        let color_index_2 = prng.random_range(0..COLORS_TOTAL);
        Self {
            color_index_1,
            color_index_2,
            position_1: 0,
            position_2: LEDS_TOTAL - 1,
            prng,
        }
    }

    pub(crate) async fn render(
        &mut self, data: &LedData,
        leds_section_1: &mut impl SmartLedsWrite<Color = RGB8, Error = impl Debug>,
        leds_section_2: &mut impl SmartLedsWrite<Color = RGB8, Error = impl Debug>,
        delay: &mut impl DelayNs, settings: &Settings,
    ) {
        let leds_section_1_future = async {
            leds_section_1
                .write(brightness(
                    gamma(data[LEDS_SECTION_1_RANGE].iter().copied()),
                    settings.brightness_damped(BRIGHTNESS_DAMPING_FACTOR),
                ))
                .unwrap();
        };

        let leds_section_2_future = async {
            leds_section_2
                .write(brightness(
                    gamma(data[LEDS_SECTION_2_RANGE].iter().copied()),
                    settings.brightness_damped(BRIGHTNESS_DAMPING_FACTOR),
                ))
                .unwrap();
        };

        join(leds_section_1_future, leds_section_2_future).await;

        delay.delay_ms(settings.delay()).await;
    }

    pub(crate) fn update(&mut self, data: &mut LedData) {
        data[self.position_1] = COLORS[self.color_index_1];
        data[self.position_2] = COLORS[self.color_index_2];

        self.position_1 += 1;
        if self.position_1 >= LEDS_PER_CARROUSEL {
            self.position_1 = 0;
            let mut new_color = self.prng.random_range(0..COLORS_TOTAL);
            while self.color_index_1 == new_color {
                // Make sure the new color is different from the current color
                new_color = self.prng.random_range(0..COLORS_TOTAL);
            }
            self.color_index_1 = new_color;
        }

        self.position_2 -= 1;
        if self.position_2 <= LEDS_PER_CARROUSEL {
            self.position_2 = LEDS_TOTAL - 1;
            let mut new_color = self.prng.random_range(0..COLORS_TOTAL);
            while self.color_index_2 == new_color {
                // Make sure the new color is different from the current color
                new_color = self.prng.random_range(0..COLORS_TOTAL);
            }
            self.color_index_2 = new_color;
        }
    }
}
