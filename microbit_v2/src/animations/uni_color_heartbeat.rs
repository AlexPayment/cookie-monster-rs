use crate::animations;
use crate::animations::{Animation, COLORS, NUM_LEDS, Settings, UniColorHeartbeat};
use core::cell::RefCell;
use embedded_hal::delay::DelayNs;
use microbit::hal::Timer;
use microbit::hal::spi::Spi;
use microbit::pac::{SPI0, TIMER0};
use smart_leds::RGB8;
use smart_leds_trait::SmartLedsWrite;
use ws2812_spi::Ws2812;

const STEP: u8 = 10;

impl<'a> UniColorHeartbeat<'a> {
    pub(crate) fn new(data: &'a RefCell<[RGB8; NUM_LEDS]>) -> Self {
        Self {
            data,
            current_step: 0,
            sequence: 0,
        }
    }
}

impl Animation for UniColorHeartbeat<'_> {
    fn brightness(&self, settings: &Settings) -> f32 {
        settings.brightness * 0.05
    }

    fn render(
        &mut self, ws2812: &mut Ws2812<Spi<SPI0>>, timer: &mut Timer<TIMER0>, settings: &Settings,
    ) {
        let brightness = (self.brightness(settings) / STEP as f32) * self.current_step as f32;
        let color =
            animations::create_color_with_brightness(&COLORS[settings.color_index], brightness);
        for i in 0..NUM_LEDS {
            self.data.borrow_mut()[i] = color;
        }

        match self.sequence {
            0 => {
                self.current_step += 1;
                if self.current_step >= STEP {
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
                if self.current_step >= STEP {
                    self.sequence = 3;
                }
            }
            3 => {
                self.current_step -= 1;
                if self.current_step == 0 {
                    self.sequence = 0;
                }
            }
            _ => {}
        }

        ws2812.write(self.data.borrow().iter().cloned()).unwrap();
        match self.sequence {
            0..=2 => timer.delay_ms(settings.delay),
            3 => timer.delay_ms(settings.delay * 30),
            _ => timer.delay_ms(settings.delay),
        }
    }

    fn reset(&mut self) {
        animations::reset_data(self.data);
        self.current_step = 0;
        self.sequence = 0;
    }
}
