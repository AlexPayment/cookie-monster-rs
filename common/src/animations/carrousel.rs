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
                    self.brightness(settings),
                ))
                .unwrap();
        };

        let leds_section_2_future = async {
            leds_section_2
                .write(brightness(
                    gamma(data[LEDS_SECTION_2_RANGE].iter().copied()),
                    self.brightness(settings),
                ))
                .unwrap();
        };

        join(leds_section_1_future, leds_section_2_future).await;

        delay.delay_ms(settings.delay()).await;
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

    fn brightness(&self, settings: &Settings) -> u8 {
        (f32::from(settings.brightness()) * 0.05) as u8
    }
}
