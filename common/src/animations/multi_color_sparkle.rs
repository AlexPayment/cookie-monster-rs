use crate::animations;
use crate::animations::{
    DELAY_SHORTEST, LEDS_SECTION_1_RANGE, LEDS_SECTION_2_RANGE, LEDS_TOTAL, LedData, Settings,
    brightness_correct, gamma_correct,
};
use core::cmp;
use core::fmt::Debug;
use embassy_futures::join::join;
use embedded_hal_async::delay::DelayNs;
use rand::rngs::SmallRng;
use rand::{RngExt, SeedableRng};
use smart_leds::RGB8;
use smart_leds_trait::SmartLedsWrite;

pub struct MultiColorSparkle {
    prng: SmallRng,
}

impl MultiColorSparkle {
    pub(crate) fn new(random_seed: u64) -> Self {
        Self {
            prng: SmallRng::seed_from_u64(random_seed),
        }
    }

    pub(crate) async fn render(
        &mut self, data: &LedData,
        leds_section_1: &mut impl SmartLedsWrite<Color = RGB8, Error = impl Debug>,
        leds_section_2: &mut impl SmartLedsWrite<Color = RGB8, Error = impl Debug>,
        delay: &mut impl DelayNs, settings: &Settings,
    ) {
        let random_delay = self
            .prng
            .random_range(DELAY_SHORTEST..cmp::max(settings.delay(), DELAY_SHORTEST + 1));

        let leds_section_1_future = async {
            // We're not using the smart_leds::brightness and smart_leds::gamma functions here
            // because not all LEDs have the same brightness.
            leds_section_1
                .write(data[LEDS_SECTION_1_RANGE].iter().copied())
                .unwrap();
        };

        let leds_section_2_future = async {
            // We're not using the smart_leds::brightness and smart_leds::gamma functions here
            // because not all LEDs have the same brightness.
            leds_section_2
                .write(data[LEDS_SECTION_2_RANGE].iter().copied())
                .unwrap();
        };

        join(leds_section_1_future, leds_section_2_future).await;

        delay.delay_ms(random_delay).await;
    }

    pub(crate) fn update(&mut self, data: &mut LedData, settings: &Settings) {
        animations::reset_data(data);

        // The number of sparkles, up to 10% of the total number of LEDs
        let sparkle_amount = self.prng.random_range(0..(LEDS_TOTAL / 10));
        for _ in 0..sparkle_amount {
            let index = self.prng.random_range(0..LEDS_TOTAL);
            // Random brightness between 0% and the set brightness
            let brightness = self.prng.random_range(0..=self.brightness(settings));
            let random_color = RGB8::new(
                self.prng.random_range(0..255),
                self.prng.random_range(0..255),
                self.prng.random_range(0..255),
            );
            data[index] = brightness_correct(gamma_correct(random_color), brightness);
        }
    }

    fn brightness(&self, settings: &Settings) -> u8 {
        settings.brightness()
    }
}
