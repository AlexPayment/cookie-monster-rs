use crate::animations::{Animation, MultiColorSolid};
use cookie_monster_common::animations;
use cookie_monster_common::animations::{LedData, NUM_COLORS, NUM_LEDS, Settings};
use embedded_hal::delay::DelayNs;
use microbit::hal::Timer;
use microbit::hal::spi::Spi;
use microbit::pac::{SPI0, TIMER0};
use smart_leds_trait::SmartLedsWrite;
use ws2812_spi::Ws2812;

const LEDS_PER_COLOR: usize = NUM_LEDS / NUM_COLORS;

impl<'a> MultiColorSolid<'a> {
    pub(crate) fn new(data: &'a LedData) -> Self {
        Self { data }
    }
}

impl Animation for MultiColorSolid<'_> {
    fn brightness(&self, settings: &Settings) -> f32 {
        settings.brightness() * 0.05
    }

    fn render(
        &mut self, ws2812: &mut Ws2812<Spi<SPI0>>, timer: &mut Timer<TIMER0>, settings: &Settings,
    ) {
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

        ws2812.write(self.data.borrow().iter().copied()).unwrap();
        // Delay from the settings doesn't really matter for the solid animations. So just using a
        // 1-second delay.
        timer.delay_ms(1_000u32);
    }

    fn reset(&mut self) {
        animations::reset_data(self.data);
    }
}
