use crate::animations;
use crate::animations::{Animation, MultiColorSolid, Settings, NUM_COLORS, NUM_LEDS};
use core::cell::RefCell;
use microbit::hal::gpio::{Output, Pin, PushPull};
use microbit::hal::prelude::_embedded_hal_blocking_delay_DelayMs;
use microbit::hal::spi::Spi;
use microbit::hal::timer::Periodic;
use microbit::hal::Timer;
use microbit::pac::{SPI0, TIMER0, TIMER1, TIMER2, TIMER3};
use smart_leds::{SmartLedsWrite, RGB8};
use ws2812_spi::Ws2812;

impl<'a> MultiColorSolid<'a> {
    pub(crate) fn new(data: &'a RefCell<[RGB8; NUM_LEDS]>) -> Self {
        MultiColorSolid { data }
    }
}

impl Animation for MultiColorSolid<'_> {
    fn render_spi(
        &mut self, ws2812: &mut Ws2812<Spi<SPI0>>, timer: &mut Timer<TIMER0>, settings: &Settings,
    ) {
        animations::reset_data(self.data);

        let leds_per_color = NUM_LEDS / NUM_COLORS;
        let mut color_index = 0;

        for i in 0..NUM_LEDS {
            if i % leds_per_color == 0 {
                color_index += 1;
            }
            self.data.borrow_mut()[i] = animations::create_color_with_brightness(
                &animations::COLORS[color_index % NUM_COLORS],
                &settings.brightness,
            );
        }

        ws2812.write(self.data.borrow().iter().cloned()).unwrap();
        // Delay from the settings doesn't really matter for the solid animations. So just using a
        // 1-second delay.
        timer.delay_ms(1_000u16);
    }

    fn render_timer(
        &mut self,
        ws2812_strip1: &mut ws2812_timer_delay::Ws2812<
            Timer<TIMER1, Periodic>,
            Pin<Output<PushPull>>,
        >,
        ws2812_strip2: &mut ws2812_timer_delay::Ws2812<
            Timer<TIMER2, Periodic>,
            Pin<Output<PushPull>>,
        >,
        ws2812_strip3: &mut ws2812_timer_delay::Ws2812<
            Timer<TIMER3, Periodic>,
            Pin<Output<PushPull>>,
        >,
        timer: &mut Timer<TIMER0>, settings: &Settings,
    ) {
        todo!()
    }
}
