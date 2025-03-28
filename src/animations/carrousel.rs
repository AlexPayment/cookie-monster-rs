use crate::animations;
use crate::animations::{Animation, Carrousel, NUM_COLORS, NUM_LEDS, Settings};
use core::cell::RefCell;
use embedded_hal::delay::DelayNs;
use microbit::hal::Timer;
use microbit::hal::spi::Spi;
use microbit::pac::{SPI0, TIMER0};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use smart_leds::RGB8;
use smart_leds_trait::SmartLedsWrite;
use ws2812_spi::Ws2812;

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

impl Animation for Carrousel<'_> {
    fn brightness(&self, settings: &Settings) -> f32 {
        settings.brightness * 0.05
    }

    fn render(
        &mut self, ws2812: &mut Ws2812<Spi<SPI0>>, timer: &mut Timer<TIMER0>, settings: &Settings,
    ) {
        self.data.borrow_mut()[self.position] = animations::create_color_with_brightness(
            &animations::COLORS[self.color_index],
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

        ws2812.write(self.data.borrow().iter().cloned()).unwrap();
        timer.delay_ms(settings.delay);
    }

    fn reset(&mut self) {
        animations::reset_data(self.data);
        self.position = 0;
    }
}
