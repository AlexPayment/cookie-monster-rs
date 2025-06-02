use crate::animations;
use crate::animations::{COLORS, LedData, NUM_LEDS, Settings};
use embedded_hal::delay::DelayNs;
use embedded_hal::spi;
use smart_leds::RGB8;
use smart_leds_trait::SmartLedsWrite;

const STEP: u8 = 10;

pub struct UniColorHeartbeat<'a> {
    data: &'a LedData,
    current_step: u8,
    sequence: u8,
}

impl<'a> UniColorHeartbeat<'a> {
    pub(crate) fn new(data: &'a LedData) -> Self {
        Self {
            data,
            current_step: 0,
            sequence: 0,
        }
    }

    pub(crate) fn render(
        &mut self, ws2812: &mut impl SmartLedsWrite<Color = RGB8, Error = impl spi::Error>,
        delay: &mut impl DelayNs, settings: &Settings,
    ) {
        ws2812.write(self.data.borrow().iter().copied()).unwrap();
        match self.sequence {
            3 => delay.delay_ms(settings.delay() * 30),
            _ => delay.delay_ms(settings.delay()),
        }
    }

    pub(crate) fn reset(&mut self) {
        animations::reset_data(self.data);
        self.current_step = 0;
        self.sequence = 0;
    }

    pub(crate) fn update(&mut self, settings: &Settings) {
        let brightness =
            (self.brightness(settings) / f32::from(STEP)) * f32::from(self.current_step);
        let color =
            animations::create_color_with_brightness(COLORS[settings.color_index()], brightness);
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
    }

    fn brightness(&self, settings: &Settings) -> f32 {
        settings.brightness() * 0.05
    }
}
