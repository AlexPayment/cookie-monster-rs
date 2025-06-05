use crate::animations;
use crate::animations::{COLORS, LedData, Settings};
use embedded_hal::spi::Error as SpiError;
use embedded_hal_async::delay::DelayNs;
use smart_leds::{RGB8, brightness, gamma};
use smart_leds_trait::SmartLedsWrite;

pub struct UniColorSolid<'a> {
    data: &'a LedData,
}

impl<'a> UniColorSolid<'a> {
    pub(crate) fn new(data: &'a LedData) -> Self {
        Self { data }
    }

    pub(crate) async fn render<E>(
        &mut self,
        ws2812: &mut impl SmartLedsWrite<Color = RGB8, Error = ws2812_spi::prerendered::Error<E>>,
        delay: &mut impl DelayNs, settings: &Settings,
    ) where
        E: SpiError,
    {
        ws2812
            .write(brightness(
                gamma(self.data.borrow().iter().copied()),
                self.brightness(settings),
            ))
            .unwrap();

        // Delay from the settings doesn't really matter for the solid animations. So just using a
        // 1-second delay.
        delay.delay_ms(1_000u32).await;
    }

    pub(crate) fn reset(&mut self) {
        animations::reset_data(self.data);
    }

    pub(crate) fn update(&mut self, settings: &Settings) {
        self.data.borrow_mut().iter_mut().for_each(|e| {
            *e = COLORS[settings.color_index()];
        });
    }

    fn brightness(&self, settings: &Settings) -> u8 {
        (f32::from(settings.brightness()) * 0.05) as u8
    }
}
