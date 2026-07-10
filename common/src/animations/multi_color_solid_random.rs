use crate::animations::{LedData, NUM_LEDS, Settings};
use core::fmt::Debug;
use embedded_hal_async::delay::DelayNs;
use rand::rngs::SmallRng;
use rand::{RngExt, SeedableRng};
use smart_leds::{RGB8, brightness, gamma};
use smart_leds_trait::SmartLedsWrite;

pub struct MultiColorSolidRandom {
    prng: SmallRng,
    rendered_data: [RGB8; NUM_LEDS],
}

impl MultiColorSolidRandom {
    pub(crate) fn new(random_seed: u64) -> Self {
        let mut animation = Self {
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

    pub(crate) async fn render(
        &mut self, data: &LedData,
        ws2812: &mut impl SmartLedsWrite<Color = RGB8, Error = impl Debug>,
        delay: &mut impl DelayNs, settings: &Settings,
    ) {
        ws2812
            .write(brightness(
                gamma(data.iter().copied()),
                self.brightness(settings),
            ))
            .unwrap();

        // Delay from the settings doesn't really matter for the solid animations. So just using a
        // 1-second delay.
        delay.delay_ms(1_000u32).await;
    }

    pub(crate) fn update(&mut self, data: &mut LedData) {
        data.copy_from_slice(&self.rendered_data);
    }

    fn brightness(&self, settings: &Settings) -> u8 {
        (f32::from(settings.brightness()) * 0.2) as u8
    }
}
