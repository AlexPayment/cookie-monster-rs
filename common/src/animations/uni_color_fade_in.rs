use crate::animations;
use crate::animations::{COLORS, LedData, Settings};
use core::fmt::Debug;
use embedded_hal_async::delay::DelayNs;
use smart_leds::{RGB8, gamma};
use smart_leds_trait::SmartLedsWrite;

const STEP: u8 = 23;

pub struct UniColorFadeIn {
    ascending: bool,
    current_step: u8,
}

impl UniColorFadeIn {
    pub(crate) fn new() -> Self {
        Self {
            ascending: true,
            current_step: 0,
        }
    }

    pub(crate) async fn render(
        &mut self, data: &LedData,
        ws2812: &mut impl SmartLedsWrite<Color = RGB8, Error = impl Debug>,
        delay: &mut impl DelayNs, settings: &Settings,
    ) {
        let brightness: u8 = (f32::from(self.brightness(settings)) * f32::from(self.current_step)
            / f32::from(STEP)) as u8;

        ws2812
            .write(smart_leds::brightness(
                gamma(data.iter().copied()),
                brightness,
            ))
            .unwrap();

        delay.delay_ms(settings.delay()).await;
    }

    pub(crate) fn update(&mut self, data: &mut LedData, settings: &Settings) {
        animations::reset_data(data);

        for led in data {
            *led = COLORS[settings.color_index()];
        }

        if self.ascending {
            self.current_step += 1;
            if self.current_step >= STEP {
                self.ascending = false;
            }
        } else {
            self.current_step -= 1;
            if self.current_step == 1 {
                self.ascending = true;
            }
        }
    }

    fn brightness(&self, settings: &Settings) -> u8 {
        (f32::from(settings.brightness()) * 0.05) as u8
    }
}
