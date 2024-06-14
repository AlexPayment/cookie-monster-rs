use crate::animations;
use crate::animations::{Animation, Carrousel, Settings, NUM_COLORS, NUM_LEDS};
use core::cell::RefCell;
use embedded_hal::delay::DelayNs;
use microbit::pac::{SPI0, TIMER0};
use nrf_hal_common::spi::Spi;
use nrf_hal_common::Timer;
use rand::prelude::SmallRng;
use rand::{Rng, SeedableRng};
use smart_leds::RGB8;
use smart_leds_trait::SmartLedsWrite;
use ws2812_spi::Ws2812;

impl<'a> Carrousel<'a> {
    pub fn new(data: &'a RefCell<[RGB8; NUM_LEDS]>, random_seed: u64) -> Self {
        let mut carrousel = Self {
            color_index: 0,
            data,
            position: 0,
            prng: SmallRng::seed_from_u64(random_seed),
        };

        carrousel.color_index = carrousel.prng.gen_range(0..NUM_COLORS);

        carrousel
    }
}

impl Animation for Carrousel<'_> {
    fn render(
        &mut self, ws2812: &mut Ws2812<Spi<SPI0>>, timer: &mut Timer<TIMER0>, settings: &Settings,
    ) {
        self.data.borrow_mut()[self.position] = animations::create_color_with_brightness(
            &animations::COLORS[self.color_index],
            &settings.brightness,
        );

        self.position += 1;
        if self.position >= NUM_LEDS {
            self.position = 0;
            self.color_index = self.prng.gen_range(0..NUM_COLORS);
        }

        ws2812.write(self.data.borrow().iter().cloned()).unwrap();
        timer.delay_ms(settings.delay);
    }

    fn reset(&mut self) {
        animations::reset_data(self.data);
        self.position = 0;
    }
}
