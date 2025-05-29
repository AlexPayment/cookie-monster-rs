use crate::animations::{Animation, DoubleCarrousel};
use cookie_monster_common::animations;
use cookie_monster_common::animations::{LedData, NUM_COLORS, NUM_LEDS, Settings};
use embedded_hal::delay::DelayNs;
use microbit::hal::Timer;
use microbit::hal::spi::Spi;
use microbit::pac::{SPI0, TIMER0};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use smart_leds_trait::SmartLedsWrite;
use ws2812_spi::Ws2812;

const LEDS_PER_CARROUSEL: usize = NUM_LEDS / 2;

impl<'a> DoubleCarrousel<'a> {
    pub fn new(data: &'a LedData, random_seed: u64) -> Self {
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
}

impl Animation for DoubleCarrousel<'_> {
    fn brightness(&self, settings: &Settings) -> f32 {
        settings.brightness() * 0.05
    }

    fn render(
        &mut self, ws2812: &mut Ws2812<Spi<SPI0>>, timer: &mut Timer<TIMER0>, settings: &Settings,
    ) {
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

        ws2812.write(self.data.borrow().iter().copied()).unwrap();
        timer.delay_ms(settings.delay());
    }

    fn reset(&mut self) {
        animations::reset_data(self.data);
        self.color_index_1 = self.prng.random_range(0..NUM_COLORS);
        self.color_index_2 = self.prng.random_range(0..NUM_COLORS);
        self.position_1 = 0;
        self.position_2 = NUM_LEDS - 1;
    }
}
