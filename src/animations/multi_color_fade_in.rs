use crate::animations;
use crate::animations::{Animation, MultiColorFadeIn, Settings, COLORS, NUM_COLORS, NUM_LEDS};
use core::cell::RefCell;
use embedded_hal::delay::DelayNs;
use microbit::pac::{SPI0, TIMER0};
use microbit::hal::spi::Spi;
use microbit::hal::Timer;
use rand::prelude::SmallRng;
use rand::{Rng, SeedableRng};
use smart_leds::RGB8;
use smart_leds_trait::SmartLedsWrite;
use ws2812_spi::Ws2812;

const STEP: u8 = 23;

impl<'a> MultiColorFadeIn<'a> {
    pub(crate) fn new(data: &'a RefCell<[RGB8; NUM_LEDS]>, random_seed: u64) -> Self {
        let mut prng = SmallRng::seed_from_u64(random_seed);
        Self {
            data,
            ascending: true,
            color_index: prng.gen_range(0..NUM_COLORS),
            prng,
            current_step: 0,
        }
    }
}

impl Animation for MultiColorFadeIn<'_> {
    fn brightness(&self, settings: &Settings) -> f32 {
        settings.brightness * 0.05
    }

    fn render(
        &mut self, ws2812: &mut Ws2812<Spi<SPI0>>, timer: &mut Timer<TIMER0>, settings: &Settings,
    ) {
        let brightness = (self.brightness(settings) / STEP as f32) * self.current_step as f32;
        let color = animations::create_color_with_brightness(&COLORS[self.color_index], brightness);
        for i in 0..NUM_LEDS {
            self.data.borrow_mut()[i] = color;
        }
        if self.ascending {
            self.current_step += 1;
            if self.current_step >= STEP {
                self.ascending = false;
            }
        } else {
            self.current_step -= 1;
            if self.current_step == 1 {
                self.color_index = self.prng.gen_range(0..NUM_COLORS);
                self.ascending = true;
            }
        }

        ws2812.write(self.data.borrow().iter().cloned()).unwrap();
        timer.delay_ms(settings.delay);
    }

    fn reset(&mut self) {
        animations::reset_data(self.data);
        self.ascending = true;
        self.current_step = 0;
    }
}
