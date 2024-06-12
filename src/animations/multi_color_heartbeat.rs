use crate::animations;
use crate::animations::{Animation, MultiColorHeartbeat, Settings, COLORS, NUM_LEDS};
use core::cell::RefCell;
use embedded_hal::delay::DelayNs;
use microbit::pac::{SPI0, TIMER0};
use nrf_hal_common::spi::Spi;
use nrf_hal_common::Timer;
use smart_leds::{RGB, RGB8};
use smart_leds_trait::SmartLedsWrite;
use ws2812_spi::Ws2812;

impl<'a> MultiColorHeartbeat<'a> {
    pub(crate) fn new(data: &'a RefCell<[RGB8; NUM_LEDS]>) -> Self {
        MultiColorHeartbeat {
            data,
            color_index: 0,
            current_step: 0,
            sequence: 0,
            step: 10,
        }
    }
}

impl Animation for MultiColorHeartbeat<'_> {
    fn render(
        &mut self, ws2812: &mut Ws2812<Spi<SPI0>>, timer: &mut Timer<TIMER0>, settings: &Settings,
    ) {
        animations::reset_data(self.data);

        let brightness = (settings.brightness / self.step as f32) * self.current_step as f32;
        let color =
            animations::create_color_with_brightness(&COLORS[self.color_index], &brightness);
        for i in 0..NUM_LEDS {
            self.data.borrow_mut()[i] = color;
        }

        match self.sequence {
            0 => {
                self.current_step += 1;
                if self.current_step >= self.step {
                    self.sequence = 1;
                }
            }
            1 => {
                self.current_step -= 1;
                if self.current_step == 1 {
                    self.sequence = 2;
                }
            }
            2 => {
                self.current_step += 1;
                if self.current_step >= self.step {
                    self.sequence = 3;
                }
            }
            3 => {
                self.current_step -= 1;
                if self.current_step == 0 {
                    self.color_index = (self.color_index + 1) % COLORS.len();
                    self.sequence = 0;
                }
            }
            _ => {}
        }

        ws2812.write(self.data.borrow().iter().cloned()).unwrap();
        match self.sequence {
            0 | 1 | 2 => timer.delay_ms(settings.delay),
            3 => timer.delay_ms(settings.delay * 30),
            _ => timer.delay_ms(settings.delay),
        }
    }
}
