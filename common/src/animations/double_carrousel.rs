use crate::animations;
use crate::animations::{LedData, NUM_COLORS, NUM_LEDS, Settings};
use embedded_hal::spi::Error as SpiError;
use embedded_hal_async::delay::DelayNs;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use smart_leds::RGB8;
use smart_leds_trait::SmartLedsWrite;

const LEDS_PER_CARROUSEL: usize = NUM_LEDS / 2;

pub struct DoubleCarrousel<'a> {
    color_index_1: usize,
    color_index_2: usize,
    data: &'a LedData,
    position_1: usize,
    position_2: usize,
    prng: SmallRng,
}

impl<'a> DoubleCarrousel<'a> {
    pub(crate) fn new(data: &'a LedData, random_seed: u64) -> Self {
        let mut prng = SmallRng::seed_from_u64(random_seed);
        let color_index_1 = prng.random_range(0..NUM_COLORS);
        let color_index_2 = prng.random_range(0..NUM_COLORS);
        Self {
            color_index_1,
            color_index_2,
            data,
            position_1: 0,
            position_2: NUM_LEDS - 1,
            prng,
        }
    }

    pub(crate) async fn render(
        &mut self, ws2812: &mut impl SmartLedsWrite<Color = RGB8, Error = impl SpiError>,
        delay: &mut impl DelayNs, settings: &Settings,
    ) {
        ws2812.write(self.data.borrow().iter().copied()).unwrap();
        delay.delay_ms(settings.delay()).await;
    }

    pub(crate) fn reset(&mut self) {
        animations::reset_data(self.data);
        self.color_index_1 = self.prng.random_range(0..NUM_COLORS);
        self.color_index_2 = self.prng.random_range(0..NUM_COLORS);
        self.position_1 = 0;
        self.position_2 = NUM_LEDS - 1;
    }

    pub(crate) fn update(&mut self, settings: &Settings) {
        self.data.borrow_mut()[self.position_1] = animations::create_color_with_brightness(
            animations::COLORS[self.color_index_1],
            self.brightness(settings),
        );
        self.data.borrow_mut()[self.position_2] = animations::create_color_with_brightness(
            animations::COLORS[self.color_index_2],
            self.brightness(settings),
        );

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

    fn brightness(&self, settings: &Settings) -> f32 {
        settings.brightness() * 0.05
    }
}
