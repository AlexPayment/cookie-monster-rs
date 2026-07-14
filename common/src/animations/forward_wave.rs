use crate::animations;
use crate::animations::{
    COLORS, LEDS_SECTION_1_RANGE, LEDS_SECTION_2_RANGE, LEDS_TOTAL, LedData, Settings,
    brightness_correct, gamma_correct,
};
use core::fmt::Debug;
use embassy_futures::join::join;
use embedded_hal_async::delay::DelayNs;
use smart_leds::RGB8;
use smart_leds_trait::SmartLedsWrite;

const WAVE_LENGTH: usize = 15;
const WAVE_SECTION_LENGTH: usize = WAVE_LENGTH / 5;

pub struct ForwardWave {
    position: usize,
    wrapped: bool,
}

impl ForwardWave {
    pub(crate) fn new() -> Self {
        Self {
            position: 0,
            wrapped: false,
        }
    }

    pub(crate) async fn render(
        &mut self, data: &LedData,
        leds_section_1: &mut impl SmartLedsWrite<Color = RGB8, Error = impl Debug>,
        leds_section_2: &mut impl SmartLedsWrite<Color = RGB8, Error = impl Debug>,
        delay: &mut impl DelayNs, settings: &Settings,
    ) {
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

        delay.delay_ms(settings.delay()).await;
    }

    pub(crate) fn update(&mut self, data: &mut LedData, settings: &Settings) {
        animations::reset_data(data);

        let wave = self.get_wave(settings);

        for (i, item) in wave.iter().enumerate() {
            let led_index = self.position as isize - i as isize;
            if self.wrapped {
                if led_index < 0 {
                    data[(LEDS_TOTAL as isize + led_index) as usize] =
                        brightness_correct(gamma_correct(COLORS[settings.color_index()]), *item);
                } else {
                    data[led_index as usize] =
                        brightness_correct(gamma_correct(COLORS[settings.color_index()]), *item);
                }
            } else if led_index >= 0 {
                data[led_index as usize] =
                    brightness_correct(gamma_correct(COLORS[settings.color_index()]), *item);
            }
        }

        self.position += 1;
        if self.position >= LEDS_TOTAL {
            self.position = 0;
            self.wrapped = true;
        }
    }

    fn brightness(&self, settings: &Settings) -> u8 {
        settings.brightness()
    }

    fn get_wave(&self, settings: &Settings) -> [u8; WAVE_LENGTH] {
        let mut wave = [0; WAVE_LENGTH];

        wave[0..WAVE_SECTION_LENGTH].iter_mut().for_each(|item| {
            *item = self.brightness(settings) / 10;
        });
        wave[WAVE_SECTION_LENGTH..(2 * WAVE_SECTION_LENGTH)]
            .iter_mut()
            .for_each(|item| {
                *item = self.brightness(settings);
            });
        wave[(2 * WAVE_SECTION_LENGTH)..(3 * WAVE_SECTION_LENGTH)]
            .iter_mut()
            .for_each(|item| {
                *item = self.brightness(settings) / 4;
            });
        wave[(3 * WAVE_SECTION_LENGTH)..(4 * WAVE_SECTION_LENGTH)]
            .iter_mut()
            .for_each(|item| {
                *item = self.brightness(settings) / 6;
            });
        wave[(4 * WAVE_SECTION_LENGTH)..WAVE_LENGTH]
            .iter_mut()
            .for_each(|item| {
                *item = self.brightness(settings) / 10;
            });

        wave
    }
}
