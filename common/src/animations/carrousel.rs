use crate::animations;
use crate::animations::{COLORS, LedData, NUM_COLORS, NUM_LEDS, Settings};
use embedded_hal::spi::Error as SpiError;
use embedded_hal_async::delay::DelayNs;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use smart_leds::{RGB8, brightness, gamma};
use smart_leds_trait::SmartLedsWrite;

pub struct Carrousel<'a> {
    color_index: usize,
    data: &'a LedData,
    position: usize,
    prng: SmallRng,
}

impl<'a> Carrousel<'a> {
    pub(crate) fn new(data: &'a LedData, random_seed: u64) -> Self {
        let mut prng = SmallRng::seed_from_u64(random_seed);
        let color_index = prng.random_range(0..NUM_COLORS);
        Self {
            color_index,
            data,
            position: 0,
            prng,
        }
    }

    pub(crate) async fn render<E>(
        &mut self,
        ws2812: &mut impl SmartLedsWrite<Color = RGB8, Error = ws2812_spi::prerendered::Error<E>>,
        delay: &mut impl DelayNs, settings: &Settings,
    ) where
        E: SpiError,
    {
        ws2812
            .write(brightness(
                gamma(self.data.borrow().iter().copied()),
                self.brightness(settings),
            ))
            .unwrap();

        delay.delay_ms(settings.delay()).await;
    }

    pub(crate) fn reset(&mut self) {
        animations::reset_data(self.data);
        self.position = 0;
    }

    pub(crate) fn update(&mut self) {
        self.data.borrow_mut()[self.position] = COLORS[self.color_index];

        self.position += 1;
        if self.position >= NUM_LEDS {
            self.position = 0;
            let mut new_color = self.prng.random_range(0..NUM_COLORS);
            while self.color_index == new_color {
                // Make sure the new color is different from the current color
                new_color = self.prng.random_range(0..NUM_COLORS);
            }
            self.color_index = new_color;
        }
    }

    fn brightness(&self, settings: &Settings) -> u8 {
        (f32::from(settings.brightness()) * 0.05) as u8
    }
}
