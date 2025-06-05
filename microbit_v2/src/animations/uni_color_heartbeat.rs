use crate::animations::{Animation, UniColorHeartbeat};
use cookie_monster_common::animations;
use cookie_monster_common::animations::{COLORS, LedData, NUM_LEDS, Settings, brightness_correct};
use embedded_hal::delay::DelayNs;
use microbit::hal::Timer;
use microbit::hal::spi::Spi;
use microbit::pac::{SPI0, TIMER0};
use smart_leds_trait::SmartLedsWrite;
use ws2812_spi::Ws2812;

const STEP: u8 = 10;

impl<'a> UniColorHeartbeat<'a> {
    pub(crate) fn new(data: &'a LedData) -> Self {
        Self {
            data,
            current_step: 0,
            sequence: 0,
        }
    }
}

impl Animation for UniColorHeartbeat<'_> {
    fn brightness(&self, settings: &Settings) -> u8 {
        (f32::from(settings.brightness()) * 0.05) as u8
    }

    fn render(
        &mut self, ws2812: &mut Ws2812<Spi<SPI0>>, timer: &mut Timer<TIMER0>, settings: &Settings,
    ) {
        let brightness = (f32::from(self.brightness(settings)) * f32::from(self.current_step)
            / f32::from(STEP)) as u8;
        let color = brightness_correct(COLORS[settings.color_index()], brightness);
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

        ws2812.write(self.data.borrow().iter().copied()).unwrap();
        match self.sequence {
            3 => timer.delay_ms(settings.delay() * 30),
            _ => timer.delay_ms(settings.delay()),
        }
    }

    fn reset(&mut self) {
        animations::reset_data(self.data);
        self.current_step = 0;
        self.sequence = 0;
    }
}
