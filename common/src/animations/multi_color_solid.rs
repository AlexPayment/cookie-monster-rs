use crate::animations;
use crate::animations::{LedData, NUM_COLORS, NUM_LEDS, Settings};
use embedded_hal::delay::DelayNs;
use embedded_hal::spi;
use smart_leds::RGB8;
use smart_leds_trait::SmartLedsWrite;

const LEDS_PER_COLOR: usize = NUM_LEDS / NUM_COLORS;

pub struct MultiColorSolid<'a> {
    data: &'a LedData,
}

impl<'a> MultiColorSolid<'a> {
    pub(crate) fn new(data: &'a LedData) -> Self {
        Self { data }
    }

    pub(crate) fn render(
        &mut self, ws2812: &mut impl SmartLedsWrite<Color = RGB8, Error = impl spi::Error>,
        delay: &mut impl DelayNs,
    ) {
        ws2812.write(self.data.borrow().iter().copied()).unwrap();
        // Delay from the settings doesn't really matter for the solid animations. So just using a
        // 1-second delay.
        delay.delay_ms(1_000u32);
    }

    pub(crate) fn reset(&mut self) {
        animations::reset_data(self.data);
    }

    pub(crate) fn update(&mut self, settings: &Settings) {
        let mut color_index = 0;

        for i in 0..NUM_LEDS {
            if i % LEDS_PER_COLOR == 0 {
                color_index += 1;
            }
            self.data.borrow_mut()[i] = animations::create_color_with_brightness(
                animations::COLORS[color_index % NUM_COLORS],
                self.brightness(settings),
            );
        }
    }

    fn brightness(&self, settings: &Settings) -> f32 {
        settings.brightness() * 0.05
    }
}
