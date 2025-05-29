use crate::animations::{Animation, MultiColorHeartbeat};
use cookie_monster_common::animations;
use cookie_monster_common::animations::{COLORS, LedData, NUM_COLORS, NUM_LEDS, Settings};
use embedded_hal::delay::DelayNs;
use microbit::hal::Timer;
use microbit::hal::spi::Spi;
use microbit::pac::{SPI0, TIMER0};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use smart_leds_trait::SmartLedsWrite;
use ws2812_spi::Ws2812;

const STEP: u8 = 10;

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
}

impl Animation for MultiColorHeartbeat<'_> {
    fn brightness(&self, settings: &Settings) -> f32 {
        settings.brightness() * 0.05
    }

    fn render(
        &mut self, ws2812: &mut Ws2812<Spi<SPI0>>, timer: &mut Timer<TIMER0>, settings: &Settings,
    ) {
        let brightness =
            (self.brightness(settings) / f32::from(STEP)) * f32::from(self.current_step);
        let color = animations::create_color_with_brightness(COLORS[self.color_index], brightness);
        for i in 0..NUM_LEDS {
            self.data.borrow_mut()[i] = color;
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

        ws2812.write(self.data.borrow().iter().copied()).unwrap();
        match self.sequence {
            3 => timer.delay_ms(settings.delay() * 25),
            _ => timer.delay_ms(settings.delay()),
        }
    }

    fn reset(&mut self) {
        animations::reset_data(self.data);
        self.current_step = 0;
        self.sequence = 0;
    }
}
