use crate::animations;
use crate::animations::{Animation, Settings, UniColorSolid, COLORS, NUM_LEDS};
use core::cell::RefCell;
use embedded_hal::delay::DelayNs;
use microbit::hal::spi::Spi;
use microbit::hal::Timer;
use microbit::pac::{SPI0, TIMER0};
use smart_leds::RGB8;
use smart_leds_trait::SmartLedsWrite;
use ws2812_spi::Ws2812;

impl<'a> UniColorSolid<'a> {
    pub fn new(data: &'a RefCell<[RGB8; NUM_LEDS]>) -> Self {
        Self { data }
    }
}

impl Animation for UniColorSolid<'_> {
    fn render(
        &mut self, ws2812: &mut Ws2812<Spi<SPI0>>, timer: &mut Timer<TIMER0>, settings: &Settings,
    ) {
        animations::reset_data(self.data);

        self.data.borrow_mut().iter_mut().for_each(|e| {
            *e = animations::create_color_with_brightness(
                &COLORS[settings.color_index],
                &settings.brightness,
            )
        });

        ws2812.write(self.data.borrow().iter().cloned()).unwrap();
        // Delay from the settings doesn't really matter for the solid animations. So just using a
        // 1-second delay.
        timer.delay_ms(1_000u32);
    }
}
