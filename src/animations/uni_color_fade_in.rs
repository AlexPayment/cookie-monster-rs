use crate::animations;
use crate::animations::{Animation, Settings, UniColorFadeIn, COLORS, NUM_LEDS};
use core::cell::RefCell;
use embedded_hal::delay::DelayNs;
use microbit::hal::spi::Spi;
use microbit::hal::Timer;
use microbit::pac::{SPI0, TIMER0};
use smart_leds::RGB8;
use smart_leds_trait::SmartLedsWrite;
use ws2812_spi::Ws2812;

impl<'a> UniColorFadeIn<'a> {
    pub(crate) fn new(data: &'a RefCell<[RGB8; NUM_LEDS]>) -> Self {
        Self {
            data,
            ascending: true,
            current_step: 0,
            step: 23,
        }
    }
}

impl<'a> Animation for UniColorFadeIn<'a> {
    fn render(
        &mut self, ws2812: &mut Ws2812<Spi<SPI0>>, timer: &mut Timer<TIMER0>, settings: &Settings,
    ) {
        animations::reset_data(self.data);

        let brightness = (settings.brightness / self.step as f32) * self.current_step as f32;
        let color =
            animations::create_color_with_brightness(&COLORS[settings.color_index], &brightness);
        for i in 0..NUM_LEDS {
            self.data.borrow_mut()[i] = color;
        }
        if self.ascending {
            self.current_step += 1;
            if self.current_step >= self.step {
                self.ascending = false;
            }
        } else {
            self.current_step -= 1;
            if self.current_step == 1 {
                self.ascending = true;
            }
        }

        ws2812.write(self.data.borrow().iter().cloned()).unwrap();
        timer.delay_ms(settings.delay);
    }
}
