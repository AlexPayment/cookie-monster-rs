use crate::animations;
use crate::animations::{
    COLORS, LedData, NUM_LEDS, SHORTEST_DELAY, Settings, brightness_correct, gamma_correct,
};
use core::cmp;
use embedded_hal::spi::Error as SpiError;
use embedded_hal_async::delay::DelayNs;
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

    pub(crate) async fn render<E>(
        &mut self,
        ws2812: &mut impl SmartLedsWrite<Color = RGB8, Error = ws2812_spi::prerendered::Error<E>>,
        delay: &mut impl DelayNs, settings: &Settings,
    ) where
        E: SpiError,
    {
        let random_delay = self
            .prng
            .random_range(SHORTEST_DELAY..cmp::max(settings.delay(), SHORTEST_DELAY + 1));

        // We're not using the smart_leds::brightness and smart_leds::gamma functions here because
        // not all LEDs have the same brightness.
        ws2812.write(self.data.borrow().iter().copied()).unwrap();

        delay.delay_ms(random_delay).await;
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
            let brightness = self.prng.random_range(0..=self.brightness(settings));
            self.data.borrow_mut()[index] =
                brightness_correct(gamma_correct(COLORS[settings.color_index()]), brightness);
        }
    }

    fn brightness(&self, settings: &Settings) -> u8 {
        settings.brightness()
    }
}
