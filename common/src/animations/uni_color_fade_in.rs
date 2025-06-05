use crate::animations;
use crate::animations::{COLORS, LedData, NUM_LEDS, Settings};
use embedded_hal::spi::Error as SpiError;
use embedded_hal_async::delay::DelayNs;
use smart_leds::{RGB8, gamma};
use smart_leds_trait::SmartLedsWrite;

const STEP: u8 = 23;

pub struct UniColorFadeIn<'a> {
    data: &'a LedData,
    ascending: bool,
    current_step: u8,
}

impl<'a> UniColorFadeIn<'a> {
    pub(crate) fn new(data: &'a LedData) -> Self {
        Self {
            data,
            ascending: true,
            current_step: 0,
        }
    }

    pub(crate) async fn render<E>(
        &mut self,
        ws2812: &mut impl SmartLedsWrite<Color = RGB8, Error = ws2812_spi::prerendered::Error<E>>,
        delay: &mut impl DelayNs, settings: &Settings,
    ) where
        E: SpiError,
    {
        let brightness: u8 = (f32::from(self.brightness(settings)) * f32::from(self.current_step)
            / f32::from(STEP)) as u8;

        ws2812
            .write(smart_leds::brightness(
                gamma(self.data.borrow().iter().copied()),
                brightness,
            ))
            .unwrap();

        delay.delay_ms(settings.delay()).await;
    }

    pub(crate) fn reset(&mut self) {
        animations::reset_data(self.data);
        self.ascending = true;
        self.current_step = 0;
    }

    pub(crate) fn update(&mut self, settings: &Settings) {
        animations::reset_data(self.data);

        for i in 0..NUM_LEDS {
            self.data.borrow_mut()[i] = COLORS[settings.color_index()];
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
