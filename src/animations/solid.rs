use crate::animations;
use crate::animations::{Animation, Settings, Solid, COLORS, NUM_LEDS};
use core::cell::RefCell;
use microbit::hal::prelude::_embedded_hal_blocking_delay_DelayMs;
use microbit::hal::spi::Spi;
use microbit::hal::Timer;
use microbit::pac::{SPI0, TIMER0};
use smart_leds::{SmartLedsWrite, RGB8};
use ws2812_spi::Ws2812;

impl<'a> Solid<'a> {
    pub fn new(data: &'a RefCell<[RGB8; NUM_LEDS]>) -> Self {
        Solid { data }
    }
}

impl Animation for Solid<'_> {
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
        // Delay from the settings doesn't really matter for the solid animation. So just using a
        // 1-second delay.
        timer.delay_ms(1_000u16);
    }
}
