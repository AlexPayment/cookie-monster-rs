use crate::animations;
use crate::animations::{COLORS, LedData, Settings, VERTICAL_SLICES};
use core::fmt::Debug;
use embedded_hal_async::delay::DelayNs;
use smart_leds::{RGB8, brightness, gamma};
use smart_leds_trait::SmartLedsWrite;

pub struct UniColorFrontToBackWave {
    position: usize,
}

impl UniColorFrontToBackWave {
    pub(crate) fn new() -> Self {
        Self { position: 0 }
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
        delay.delay_ms(settings.delay()).await;
    }

    pub(crate) fn update(&mut self, data: &mut LedData, settings: &Settings) {
        animations::reset_data(data);

        let slice = &VERTICAL_SLICES[self.position];

        for led in slice {
            led.map(|l| {
                data[usize::from(l)] = COLORS[settings.color_index()];
            });
        }

        self.position = (self.position + 1) % VERTICAL_SLICES.len();
    }

    fn brightness(&self, settings: &Settings) -> u8 {
        settings.brightness()
    }
}
