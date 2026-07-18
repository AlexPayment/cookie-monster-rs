use crate::animations::{
    LEDS_SECTION_1_RANGE, LEDS_SECTION_2_RANGE, LEDS_TOTAL, LedData, Settings,
};
use core::fmt::Debug;
use embassy_futures::join::join;
use embedded_hal_async::delay::DelayNs;
use rand::rngs::SmallRng;
use rand::{RngExt, SeedableRng};
use smart_leds::hsv::{Hsv, hsv2rgb};
use smart_leds::{RGB8, brightness, gamma};
use smart_leds_trait::SmartLedsWrite;

const BRIGHTNESS_DAMPING_FACTOR: f32 = 0.2;

pub struct Shimmer {
    hsv: Hsv,
}

impl Shimmer {
    pub(crate) fn new(random_seed: u64) -> Self {
        let mut prng = SmallRng::seed_from_u64(random_seed);
        Self {
            hsv: Hsv {
                // Start the animation with a random hue.
                hue: prng.random_range(0..=u8::MAX),
                sat: 255,
                val: 255,
            },
        }
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

        delay.delay_ms(settings.delay()).await;
    }

    pub(crate) fn update(&mut self, data: &mut LedData) {
        let rgb = hsv2rgb(self.hsv);
        *data = [rgb; LEDS_TOTAL];
        self.hsv.hue = self.hsv.hue.wrapping_add(1);
    }
}
