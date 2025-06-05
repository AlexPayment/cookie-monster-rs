use crate::animations;
use crate::animations::{COLORS, LedData, NUM_COLORS, NUM_LEDS, Settings};
use embedded_hal::spi::Error as SpiError;
use embedded_hal_async::delay::DelayNs;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use smart_leds::{RGB8, gamma};
use smart_leds_trait::SmartLedsWrite;

const STEP: u8 = 10;

pub struct MultiColorHeartbeat<'a> {
    data: &'a LedData,
    color_index: usize,
    prng: SmallRng,
    current_step: u8,
    sequence: u8,
}

impl<'a> MultiColorHeartbeat<'a> {
    pub(crate) fn new(data: &'a LedData, random_seed: u64) -> Self {
        let mut prng = SmallRng::seed_from_u64(random_seed);
        Self {
            data,
            color_index: prng.random_range(0..NUM_COLORS),
            prng,
            current_step: 0,
            sequence: 0,
        }
    }

    pub(crate) async fn render<E>(
        &mut self,
        ws2812: &mut impl SmartLedsWrite<Color = RGB8, Error = ws2812_spi::prerendered::Error<E>>,
        delay: &mut impl DelayNs, settings: &Settings,
    ) where
        E: SpiError,
    {
        let brightness = (f32::from(self.brightness(settings)) * f32::from(self.current_step)
            / f32::from(STEP)) as u8;

        ws2812
            .write(smart_leds::brightness(
                gamma(self.data.borrow().iter().copied()),
                brightness,
            ))
            .unwrap();

        match self.sequence {
            3 => delay.delay_ms(settings.delay() * 25).await,
            _ => delay.delay_ms(settings.delay()).await,
        }
    }

    pub(crate) fn reset(&mut self) {
        animations::reset_data(self.data);
        self.current_step = 0;
        self.sequence = 0;
    }

    pub(crate) fn update(&mut self) {
        for i in 0..NUM_LEDS {
            self.data.borrow_mut()[i] = COLORS[self.color_index];
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
                    self.color_index = self.prng.random_range(0..NUM_COLORS);
                    self.sequence = 0;
                }
            }
            _ => {}
        }
    }

    fn brightness(&self, settings: &Settings) -> u8 {
        (f32::from(settings.brightness()) * 0.05) as u8
    }
}
