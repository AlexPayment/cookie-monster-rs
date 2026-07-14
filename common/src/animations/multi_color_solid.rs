use crate::animations::{
    COLORS, COLORS_TOTAL, LEDS_SECTION_1_RANGE, LEDS_SECTION_2_RANGE, LEDS_TOTAL, LedData, Settings,
};
use core::fmt::Debug;
use embassy_futures::join::join;
use embedded_hal_async::delay::DelayNs;
use smart_leds::{RGB8, brightness, gamma};
use smart_leds_trait::SmartLedsWrite;

const LEDS_PER_COLOR: usize = LEDS_TOTAL / COLORS_TOTAL;

pub struct MultiColorSolid {}

impl MultiColorSolid {
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
                    self.brightness(settings),
                ))
                .unwrap();
        };

        let leds_section_2_future = async {
            leds_section_2
                .write(brightness(
                    gamma(data[LEDS_SECTION_2_RANGE].iter().copied()),
                    self.brightness(settings),
                ))
                .unwrap();
        };

        join(leds_section_1_future, leds_section_2_future).await;

        // Delay from the settings doesn't really matter for the solid animations. So just using a
        // 1-second delay.
        delay.delay_ms(1_000u32).await;
    }

    pub(crate) fn update(&mut self, data: &mut LedData) {
        let mut color_index = 0;

        for (i, led) in data.iter_mut().enumerate() {
            if i % LEDS_PER_COLOR == 0 {
                color_index += 1;
            }
            *led = COLORS[color_index % COLORS_TOTAL];
        }
    }

    fn brightness(&self, settings: &Settings) -> u8 {
        (f32::from(settings.brightness()) * 0.05) as u8
    }
}
