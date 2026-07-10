use crate::animations::{COLORS, LedData, NUM_COLORS, NUM_LEDS, Settings};
use core::fmt::Debug;
use embedded_hal_async::delay::DelayNs;
use smart_leds::{RGB8, brightness, gamma};
use smart_leds_trait::SmartLedsWrite;

const LEDS_PER_COLOR: usize = NUM_LEDS / NUM_COLORS;

pub struct MultiColorSolid {}

impl MultiColorSolid {
    pub(crate) fn new() -> Self {
        Self {}
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
        let mut color_index = 0;

        for (i, led) in data.iter_mut().enumerate() {
            if i % LEDS_PER_COLOR == 0 {
                color_index += 1;
            }
            *led = COLORS[color_index % NUM_COLORS];
        }
    }

    fn brightness(&self, settings: &Settings) -> u8 {
        (f32::from(settings.brightness()) * 0.05) as u8
    }
}
