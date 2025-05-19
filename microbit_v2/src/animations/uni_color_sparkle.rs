use crate::animations;
use crate::animations::{Animation, COLORS, NUM_LEDS, SHORTEST_DELAY, Settings, UniColorSparkle};
use core::cell::RefCell;
use core::cmp;
use embedded_hal::delay::DelayNs;
use microbit::hal::Timer;
use microbit::hal::spi::Spi;
use microbit::pac::{SPI0, TIMER0};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use smart_leds::RGB8;
use smart_leds_trait::SmartLedsWrite;
use ws2812_spi::Ws2812;

impl<'a> UniColorSparkle<'a> {
    pub(crate) fn new(data: &'a RefCell<[RGB8; NUM_LEDS]>, random_seed: u64) -> Self {
        Self {
            data,
            prng: SmallRng::seed_from_u64(random_seed),
        }
    }
}

impl Animation for UniColorSparkle<'_> {
    fn brightness(&self, settings: &Settings) -> f32 {
        settings.brightness
    }

    fn render(
        &mut self, ws2812: &mut Ws2812<Spi<SPI0>>, timer: &mut Timer<TIMER0>, settings: &Settings,
    ) {
        animations::reset_data(self.data);

        // The amount of sparkles, up to 10% of the total number of LEDs
        let sparkle_amount = self.prng.random_range(0..(NUM_LEDS / 10));
        for _ in 0..sparkle_amount {
            let index = self.prng.random_range(0..NUM_LEDS);
            // Random brightness between 0% and the set brightness
            let brightness = self.prng.random_range(0.0..=self.brightness(settings));
            self.data.borrow_mut()[index] =
                animations::create_color_with_brightness(&COLORS[settings.color_index], brightness);
        }

        let random_delay = self
            .prng
            .random_range(SHORTEST_DELAY..cmp::max(settings.delay, SHORTEST_DELAY + 1));

        ws2812.write(self.data.borrow().iter().cloned()).unwrap();
        timer.delay_ms(random_delay);
    }

    fn reset(&mut self) {
        animations::reset_data(self.data);
    }
}
