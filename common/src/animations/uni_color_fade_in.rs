use crate::animations;
use crate::animations::{COLORS, LEDS_SECTION_1_RANGE, LEDS_SECTION_2_RANGE, LedData, Settings};
use core::fmt::Debug;
use embassy_futures::join::join;
use embedded_hal_async::delay::DelayNs;
use smart_leds::{RGB8, gamma};
use smart_leds_trait::SmartLedsWrite;

const STEP: u8 = 23;

pub struct UniColorFadeIn {
    ascending: bool,
    current_step: u8,
}

impl UniColorFadeIn {
    pub(crate) const BRIGHTNESS_DAMPING_FACTOR: f32 = 0.05;

    pub(crate) fn new() -> Self {
        Self {
            ascending: true,
            current_step: 0,
        }
    }

    pub(crate) async fn render(
        &mut self, data: &LedData,
        leds_section_1: &mut impl SmartLedsWrite<Color = RGB8, Error = impl Debug>,
        leds_section_2: &mut impl SmartLedsWrite<Color = RGB8, Error = impl Debug>,
        delay: &mut impl DelayNs, settings: &Settings,
    ) {
        let brightness: u8 =
            (f32::from(settings.brightness_damped(Self::BRIGHTNESS_DAMPING_FACTOR))
                * f32::from(self.current_step)
                / f32::from(STEP)) as u8;

        let leds_section_1_future = async {
            leds_section_1
                .write(smart_leds::brightness(
                    gamma(data[LEDS_SECTION_1_RANGE].iter().copied()),
                    brightness,
                ))
                .unwrap();
        };

        let leds_section_2_future = async {
            leds_section_2
                .write(smart_leds::brightness(
                    gamma(data[LEDS_SECTION_2_RANGE].iter().copied()),
                    brightness,
                ))
                .unwrap();
        };

        join(leds_section_1_future, leds_section_2_future).await;

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
}
