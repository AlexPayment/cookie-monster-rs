use crate::animations::{
    COLORS, COLORS_TOTAL, LEDS_SECTION_1_RANGE, LEDS_SECTION_2_RANGE, LedData, Settings,
};
use core::fmt::Debug;
use embassy_futures::join::join;
use embedded_hal_async::delay::DelayNs;
use rand::rngs::SmallRng;
use rand::{RngExt, SeedableRng};
use smart_leds::{RGB8, gamma};
use smart_leds_trait::SmartLedsWrite;

const STEP: u8 = 10;

pub struct MultiColorHeartbeat {
    color_index: usize,
    prng: SmallRng,
    current_step: u8,
    sequence: u8,
}

impl MultiColorHeartbeat {
    pub(crate) fn new(random_seed: u64) -> Self {
        let mut prng = SmallRng::seed_from_u64(random_seed);
        Self {
            color_index: prng.random_range(0..COLORS_TOTAL),
            prng,
            current_step: 0,
            sequence: 0,
        }
    }

    pub(crate) async fn render(
        &mut self, data: &LedData,
        leds_section_1: &mut impl SmartLedsWrite<Color = RGB8, Error = impl Debug>,
        leds_section_2: &mut impl SmartLedsWrite<Color = RGB8, Error = impl Debug>,
        delay: &mut impl DelayNs, settings: &Settings,
    ) {
        let brightness = (f32::from(settings.brightness_damped(0.05))
            * f32::from(self.current_step)
            / f32::from(STEP)) as u8;

        let leds_section_1_future = async {
            leds_section_1
                .write(smart_leds::brightness(
                    gamma(data[LEDS_SECTION_1_RANGE].iter().copied()),
                    brightness,
                ))
                .unwrap();
        };

        let leds_section_2_future = async {
            leds_section_2
                .write(smart_leds::brightness(
                    gamma(data[LEDS_SECTION_2_RANGE].iter().copied()),
                    brightness,
                ))
                .unwrap();
        };

        join(leds_section_1_future, leds_section_2_future).await;

        match self.sequence {
            3 => delay.delay_ms(settings.delay() * 25).await,
            _ => delay.delay_ms(settings.delay()).await,
        }
    }

    pub(crate) fn update(&mut self, data: &mut LedData) {
        for led in data {
            *led = COLORS[self.color_index];
        }

        match self.sequence {
            0 => {
                self.current_step += 1;
                if self.current_step >= STEP {
                    self.sequence = 1;
                }
            }
            1 => {
                self.current_step -= 1;
                if self.current_step == 1 {
                    self.sequence = 2;
                }
            }
            2 => {
                self.current_step += 1;
                if self.current_step >= STEP {
                    self.sequence = 3;
                }
            }
            3 => {
                self.current_step -= 1;
                if self.current_step == 0 {
                    self.color_index = self.prng.random_range(0..COLORS_TOTAL);
                    self.sequence = 0;
                }
            }
            _ => {}
        }
    }
}
