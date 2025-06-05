use crate::animations::{Animation, MultiColorSolidRandom};
use cookie_monster_common::animations;
use cookie_monster_common::animations::{LedData, NUM_LEDS, Settings, brightness_correct};
use embedded_hal::delay::DelayNs;
use microbit::hal::Timer;
use microbit::hal::spi::Spi;
use microbit::pac::{SPI0, TIMER0};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use smart_leds::RGB8;
use smart_leds_trait::SmartLedsWrite;
use ws2812_spi::Ws2812;

impl<'a> MultiColorSolidRandom<'a> {
    pub(crate) fn new(data: &'a LedData, random_seed: u64) -> Self {
        let mut animation = Self {
            data,
            prng: SmallRng::seed_from_u64(random_seed),
            rendered_data: [RGB8::default(); NUM_LEDS],
        };

        for i in 0..NUM_LEDS {
            let random_color = RGB8::new(
                animation.prng.random_range(0..255),
                animation.prng.random_range(0..255),
                animation.prng.random_range(0..255),
            );
            animation.rendered_data[i] = random_color;
        }

        animation
    }
}

impl Animation for MultiColorSolidRandom<'_> {
    fn brightness(&self, settings: &Settings) -> u8 {
        (f32::from(settings.brightness()) * 0.2) as u8
    }

    fn render(
        &mut self, ws2812: &mut Ws2812<Spi<SPI0>>, timer: &mut Timer<TIMER0>, settings: &Settings,
    ) {
        for i in 0..NUM_LEDS {
            self.data.borrow_mut()[i] =
                brightness_correct(self.rendered_data[i], self.brightness(settings));
        }

        ws2812.write(self.data.borrow().iter().copied()).unwrap();
        // Delay from the settings doesn't really matter for the solid animations. So just using a
        // 1-second delay.
        timer.delay_ms(1_000u32);
    }

    fn reset(&mut self) {
        animations::reset_data(self.data);
    }
}
