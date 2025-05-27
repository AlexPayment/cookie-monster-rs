use crate::animations;
use crate::animations::{Animation, COLORS, NUM_LEDS, UniColorFadeIn};
use cookie_monster_common::animations::Settings;
use core::cell::RefCell;
use embedded_hal::delay::DelayNs;
use microbit::hal::Timer;
use microbit::hal::spi::Spi;
use microbit::pac::{SPI0, TIMER0};
use smart_leds::RGB8;
use smart_leds_trait::SmartLedsWrite;
use ws2812_spi::Ws2812;

const STEP: u8 = 23;

impl<'a> UniColorFadeIn<'a> {
    pub(crate) fn new(data: &'a RefCell<[RGB8; NUM_LEDS]>) -> Self {
        Self {
            data,
            ascending: true,
            current_step: 0,
        }
    }
}

impl Animation for UniColorFadeIn<'_> {
    fn brightness(&self, settings: &Settings) -> f32 {
        settings.brightness() * 0.05
    }

    fn render(
        &mut self, ws2812: &mut Ws2812<Spi<SPI0>>, timer: &mut Timer<TIMER0>, settings: &Settings,
    ) {
        animations::reset_data(self.data);

        let brightness =
            (self.brightness(settings) / f32::from(STEP)) * f32::from(self.current_step);
        let color =
            animations::create_color_with_brightness(COLORS[settings.color_index()], brightness);
        for i in 0..NUM_LEDS {
            self.data.borrow_mut()[i] = color;
        }
        if self.ascending {
            self.current_step += 1;
            if self.current_step >= STEP {
                self.ascending = false;
            }
        } else {
            self.current_step -= 1;
            if self.current_step == 1 {
                self.ascending = true;
            }
        }

        ws2812.write(self.data.borrow().iter().copied()).unwrap();
        timer.delay_ms(settings.delay());
    }

    fn reset(&mut self) {
        animations::reset_data(self.data);
        self.ascending = true;
        self.current_step = 0;
    }
}
