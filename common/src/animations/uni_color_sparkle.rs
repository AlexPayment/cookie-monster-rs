use crate::animations;
use crate::animations::{COLORS, LedData, NUM_LEDS, SHORTEST_DELAY, Settings};
use core::cmp;
use embedded_hal::delay::DelayNs;
use embedded_hal::spi;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use smart_leds::RGB8;
use smart_leds_trait::SmartLedsWrite;

pub struct UniColorSparkle<'a> {
    data: &'a LedData,
    prng: SmallRng,
}

impl<'a> UniColorSparkle<'a> {
    pub(crate) fn new(data: &'a LedData, random_seed: u64) -> Self {
        Self {
            data,
            prng: SmallRng::seed_from_u64(random_seed),
        }
    }

    pub(crate) fn render(
        &mut self, ws2812: &mut impl SmartLedsWrite<Color = RGB8, Error = impl spi::Error>,
        delay: &mut impl DelayNs, settings: &Settings,
    ) {
        let random_delay = self
            .prng
            .random_range(SHORTEST_DELAY..cmp::max(settings.delay(), SHORTEST_DELAY + 1));

        ws2812.write(self.data.borrow().iter().copied()).unwrap();
        delay.delay_ms(random_delay);
    }

    pub(crate) fn reset(&mut self) {
        animations::reset_data(self.data);
    }

    pub(crate) fn update(&mut self, settings: &Settings) {
        animations::reset_data(self.data);

        // The amount of sparkles, up to 10% of the total number of LEDs
        let sparkle_amount = self.prng.random_range(0..(NUM_LEDS / 10));
        for _ in 0..sparkle_amount {
            let index = self.prng.random_range(0..NUM_LEDS);
            // Random brightness between 0% and the set brightness
            let brightness = self.prng.random_range(0.0..=self.brightness(settings));
            self.data.borrow_mut()[index] = animations::create_color_with_brightness(
                COLORS[settings.color_index()],
                brightness,
            );
        }
    }

    fn brightness(&self, settings: &Settings) -> f32 {
        settings.brightness()
    }
}
