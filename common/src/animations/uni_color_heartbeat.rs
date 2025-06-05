use crate::animations;
use crate::animations::{COLORS, LedData, NUM_LEDS, Settings};
use embedded_hal::spi::Error as SpiError;
use embedded_hal_async::delay::DelayNs;
use smart_leds::{RGB8, gamma};
use smart_leds_trait::SmartLedsWrite;

const STEP: u8 = 10;

pub struct UniColorHeartbeat<'a> {
    data: &'a LedData,
    current_step: u8,
    sequence: u8,
}

impl<'a> UniColorHeartbeat<'a> {
    pub(crate) fn new(data: &'a LedData) -> Self {
        Self {
            data,
            current_step: 0,
            sequence: 0,
        }
    }

    pub(crate) async fn render<E>(
        &mut self,
        ws2812: &mut impl SmartLedsWrite<Color = RGB8, Error = ws2812_spi::prerendered::Error<E>>,
        delay: &mut impl DelayNs, settings: &Settings,
    ) where
        E: SpiError,
    {
        let brightness = (f32::from(self.brightness(settings)) * f32::from(self.current_step)
            / f32::from(STEP)) as u8;

        ws2812
            .write(smart_leds::brightness(
                gamma(self.data.borrow().iter().copied()),
                brightness,
            ))
            .unwrap();

        match self.sequence {
            3 => delay.delay_ms(settings.delay() * 30).await,
            _ => delay.delay_ms(settings.delay()).await,
        }
    }

    pub(crate) fn reset(&mut self) {
        animations::reset_data(self.data);
        self.current_step = 0;
        self.sequence = 0;
    }

    pub(crate) fn update(&mut self, settings: &Settings) {
        for i in 0..NUM_LEDS {
            self.data.borrow_mut()[i] = COLORS[settings.color_index()];
        }

        match self.sequence {
            0 => {
                self.current_step += 1;
                if self.current_step >= STEP {
                    self.sequence = 1;
                }
            }
            1 => {
                self.current_step -= 1;
                if self.current_step == 1 {
                    self.sequence = 2;
                }
            }
            2 => {
                self.current_step += 1;
                if self.current_step >= STEP {
                    self.sequence = 3;
                }
            }
            3 => {
                self.current_step -= 1;
                if self.current_step == 0 {
                    self.sequence = 0;
                }
            }
            _ => {}
        }
    }

    fn brightness(&self, settings: &Settings) -> u8 {
        (f32::from(settings.brightness()) * 0.05) as u8
    }
}
