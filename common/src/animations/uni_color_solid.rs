use crate::animations::{COLORS, LEDS_SECTION_1_RANGE, LEDS_SECTION_2_RANGE, LedData, Settings};
use core::fmt::Debug;
use embassy_futures::join::join;
use embedded_hal_async::delay::DelayNs;
use smart_leds::{RGB8, brightness, gamma};
use smart_leds_trait::SmartLedsWrite;

const BRIGHTNESS_DAMPING_FACTOR: f32 = 0.05;

pub struct UniColorSolid {}

impl UniColorSolid {
    pub(crate) fn new() -> Self {
        Self {}
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

    pub(crate) fn update(&mut self, data: &mut LedData, settings: &Settings) {
        data.iter_mut().for_each(|e| {
            *e = COLORS[settings.color_index()];
        });
    }
}
