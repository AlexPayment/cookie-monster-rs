use crate::animations;
use crate::animations::{COLORS, LedData, NUM_COLORS, NUM_LEDS, Settings};
use embedded_hal::spi::Error as SpiError;
use embedded_hal_async::delay::DelayNs;
use smart_leds::{RGB8, brightness, gamma};
use smart_leds_trait::SmartLedsWrite;

const LEDS_PER_COLOR: usize = NUM_LEDS / NUM_COLORS;

pub struct MultiColorSolid<'a> {
    data: &'a LedData,
}

impl<'a> MultiColorSolid<'a> {
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

    pub(crate) fn update(&mut self) {
        let mut color_index = 0;

        for i in 0..NUM_LEDS {
            if i % LEDS_PER_COLOR == 0 {
                color_index += 1;
            }
            self.data.borrow_mut()[i] = COLORS[color_index % NUM_COLORS];
        }
    }

    fn brightness(&self, settings: &Settings) -> u8 {
        (f32::from(settings.brightness()) * 0.05) as u8
    }
}
