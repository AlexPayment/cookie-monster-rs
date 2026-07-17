use crate::animations::{
    LEDS_SECTION_1_RANGE, LEDS_SECTION_2_RANGE, LEDS_TOTAL, LedData, Settings,
};
use core::fmt::Debug;
use embassy_futures::join::join;
use embedded_hal_async::delay::DelayNs;
use rand::rngs::SmallRng;
use rand::{RngExt, SeedableRng};
use smart_leds::{RGB8, brightness, gamma};
use smart_leds_trait::SmartLedsWrite;

const BRIGHTNESS_DAMPING_FACTOR: f32 = 0.2;

pub struct MultiColorSolidRandom {
    prng: SmallRng,
    rendered_data: [RGB8; LEDS_TOTAL],
}

impl MultiColorSolidRandom {
    pub(crate) fn new(random_seed: u64) -> Self {
        let mut animation = Self {
            prng: SmallRng::seed_from_u64(random_seed),
            rendered_data: [RGB8::default(); LEDS_TOTAL],
        };

        for i in 0..LEDS_TOTAL {
            let random_color = RGB8::new(
                animation.prng.random_range(0..=u8::MAX),
                animation.prng.random_range(0..=u8::MAX),
                animation.prng.random_range(0..=u8::MAX),
            );
            animation.rendered_data[i] = random_color;
        }

        animation
    }

    pub(crate) async fn render(
        &mut self, data: &LedData,
        leds_section_1: &mut impl SmartLedsWrite<Color = RGB8, Error = impl Debug>,
        leds_section_2: &mut impl SmartLedsWrite<Color = RGB8, Error = impl Debug>,
        delay: &mut impl DelayNs, settings: &Settings,
    ) {
        let leds_section_1_future = async {
            leds_section_1
                .write(brightness(
                    gamma(data[LEDS_SECTION_1_RANGE].iter().copied()),
                    settings.brightness_damped(BRIGHTNESS_DAMPING_FACTOR),
                ))
                .unwrap();
        };

        let leds_section_2_future = async {
            leds_section_2
                .write(brightness(
                    gamma(data[LEDS_SECTION_2_RANGE].iter().copied()),
                    settings.brightness_damped(BRIGHTNESS_DAMPING_FACTOR),
                ))
                .unwrap();
        };

        join(leds_section_1_future, leds_section_2_future).await;

        // Delay from the settings doesn't really matter for the solid animations. So just using a
        // 1-second delay.
        delay.delay_ms(1_000u32).await;
    }

    pub(crate) fn update(&mut self, data: &mut LedData) {
        data.copy_from_slice(&self.rendered_data);
    }
}
