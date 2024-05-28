use crate::animations;
use crate::animations::{Animation, MultiColorSolidRandom, Settings, NUM_LEDS};
use core::cell::RefCell;
use embedded_hal::delay::DelayNs;
use microbit::hal::spi::Spi;
use microbit::hal::Timer;
use microbit::pac::{SPI0, TIMER0};
use rand::prelude::SmallRng;
use rand::{Rng, SeedableRng};
use smart_leds::RGB8;
use smart_leds_trait::SmartLedsWrite;
use ws2812_spi::Ws2812;

impl<'a> MultiColorSolidRandom<'a> {
    pub(crate) fn new(data: &'a RefCell<[RGB8; NUM_LEDS]>, random_seed: u64) -> Self {
        MultiColorSolidRandom {
            data,
            prng: SmallRng::seed_from_u64(random_seed),
            rendered: false,
            rendered_data: [RGB8::default(); NUM_LEDS],
        }
    }
}

impl Animation for MultiColorSolidRandom<'_> {
    fn render(
        &mut self, ws2812: &mut Ws2812<Spi<SPI0>>, timer: &mut Timer<TIMER0>, settings: &Settings,
    ) {
        animations::reset_data(self.data);

        if !self.rendered {
            for i in 0..NUM_LEDS {
                let random_color = RGB8::new(
                    self.prng.gen_range(0..255),
                    self.prng.gen_range(0..255),
                    self.prng.gen_range(0..255),
                );
                self.rendered_data[i] = random_color;
            }
            self.rendered = true;
        }

        for i in 0..NUM_LEDS {
            self.data.borrow_mut()[i] = animations::create_color_with_brightness(
                &self.rendered_data[i],
                &settings.brightness,
            );
        }

        ws2812.write(self.data.borrow().iter().cloned()).unwrap();
        // Delay from the settings doesn't really matter for the solid animations. So just using a
        // 1-second delay.
        timer.delay_ms(1_000u32);
    }
}
