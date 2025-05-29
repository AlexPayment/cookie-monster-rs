use crate::animations;
use crate::animations::{NUM_COLORS, NUM_LEDS, Settings};
use core::cell::RefCell;
use embedded_hal::delay::DelayNs;
use embedded_hal::spi;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use smart_leds::RGB8;
use smart_leds_trait::SmartLedsWrite;

pub struct Carrousel<'a> {
    color_index: usize,
    data: &'a RefCell<[RGB8; NUM_LEDS]>,
    position: usize,
    prng: SmallRng,
}

impl<'a> Carrousel<'a> {
    pub fn new(data: &'a RefCell<[RGB8; NUM_LEDS]>, random_seed: u64) -> Self {
        let mut prng = SmallRng::seed_from_u64(random_seed);
        let color_index = prng.random_range(0..NUM_COLORS);
        Self {
            color_index,
            data,
            position: 0,
            prng,
        }
    }
}

impl Carrousel<'_> {
    pub fn render(
        &mut self, ws2812: &mut impl SmartLedsWrite<Color = RGB8, Error = impl spi::Error>,
        delay: &mut impl DelayNs, settings: &Settings,
    ) {
        ws2812.write(self.data.borrow().iter().copied()).unwrap();
        delay.delay_ms(settings.delay());
    }

    pub fn reset(&mut self) {
        animations::reset_data(self.data);
        self.position = 0;
    }

    pub fn update(&mut self, settings: &Settings) {
        self.data.borrow_mut()[self.position] = animations::create_color_with_brightness(
            animations::COLORS[self.color_index],
            self.brightness(settings),
        );

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

    fn brightness(&self, settings: &Settings) -> f32 {
        settings.brightness() * 0.05
    }
}
