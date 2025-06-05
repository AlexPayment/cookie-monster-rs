use crate::animations;
use crate::animations::{COLORS, LedData, NUM_LEDS, Settings, brightness_correct, gamma_correct};
use embedded_hal::spi::Error as SpiError;
use embedded_hal_async::delay::DelayNs;
use smart_leds::RGB8;
use smart_leds_trait::SmartLedsWrite;

const WAVE_LENGTH: usize = 15;
const WAVE_SECTION_LENGTH: usize = WAVE_LENGTH / 5;

pub struct ForwardWave<'a> {
    data: &'a LedData,
    position: usize,
    wrapped: bool,
}

impl<'a> ForwardWave<'a> {
    pub(crate) fn new(data: &'a LedData) -> Self {
        Self {
            data,
            position: 0,
            wrapped: false,
        }
    }

    pub(crate) async fn render<E>(
        &mut self,
        ws2812: &mut impl SmartLedsWrite<Color = RGB8, Error = ws2812_spi::prerendered::Error<E>>,
        delay: &mut impl DelayNs, settings: &Settings,
    ) where
        E: SpiError,
    {
        // We're not using the smart_leds::brightness and smart_leds::gamma functions here because
        // not all LEDs have the same brightness.
        ws2812.write(self.data.borrow().iter().copied()).unwrap();

        delay.delay_ms(settings.delay()).await;
    }

    pub(crate) fn reset(&mut self) {
        animations::reset_data(self.data);
        self.position = 0;
        self.wrapped = false;
    }

    pub(crate) fn update(&mut self, settings: &Settings) {
        animations::reset_data(self.data);

        let wave = self.get_wave(settings);

        for (i, item) in wave.iter().enumerate() {
            let led_index = (self.position - i) as isize;
            if self.wrapped {
                if led_index < 0 {
                    self.data.borrow_mut()[(NUM_LEDS as isize + led_index) as usize] =
                        brightness_correct(gamma_correct(COLORS[settings.color_index()]), *item);
                } else {
                    self.data.borrow_mut()[led_index as usize] =
                        brightness_correct(gamma_correct(COLORS[settings.color_index()]), *item);
                }
            } else if led_index >= 0 {
                self.data.borrow_mut()[led_index as usize] =
                    brightness_correct(gamma_correct(COLORS[settings.color_index()]), *item);
            }
        }

        self.position += 1;
        if self.position >= NUM_LEDS {
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
