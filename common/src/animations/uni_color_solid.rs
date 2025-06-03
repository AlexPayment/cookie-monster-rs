use crate::animations;
use crate::animations::{COLORS, LedData, Settings};
use embedded_hal::spi::Error as SpiError;
use embedded_hal_async::delay::DelayNs;
use smart_leds::RGB8;
use smart_leds_trait::SmartLedsWrite;

pub struct UniColorSolid<'a> {
    data: &'a LedData,
}

impl<'a> UniColorSolid<'a> {
    pub(crate) fn new(data: &'a LedData) -> Self {
        Self { data }
    }

    pub(crate) async fn render(
        &mut self, ws2812: &mut impl SmartLedsWrite<Color = RGB8, Error = impl SpiError>,
        delay: &mut impl DelayNs,
    ) {
        ws2812.write(self.data.borrow().iter().copied()).unwrap();
        // Delay from the settings doesn't really matter for the solid animations. So just using a
        // 1-second delay.
        delay.delay_ms(1_000u32).await;
    }

    pub(crate) fn reset(&mut self) {
        animations::reset_data(self.data);
    }

    pub(crate) fn update(&mut self, settings: &Settings) {
        self.data.borrow_mut().iter_mut().for_each(|e| {
            *e = animations::create_color_with_brightness(
                COLORS[settings.color_index()],
                self.brightness(settings),
            );
        });
    }

    fn brightness(&self, settings: &Settings) -> f32 {
        settings.brightness() * 0.05
    }
}
