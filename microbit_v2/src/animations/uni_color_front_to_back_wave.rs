use crate::animations::{Animation, UniColorFrontToBackWave};
use cookie_monster_common::animations;
use cookie_monster_common::animations::{
    COLORS, LedData, Settings, VERTICAL_SLICES, brightness_correct,
};
use embedded_hal::delay::DelayNs;
use microbit::pac::{SPI0, TIMER0};
use nrf52833_hal::Timer;
use nrf52833_hal::spi::Spi;
use smart_leds_trait::SmartLedsWrite;
use ws2812_spi::Ws2812;

impl<'a> UniColorFrontToBackWave<'a> {
    pub fn new(data: &'a LedData) -> Self {
        Self { data, position: 0 }
    }
}

impl Animation for UniColorFrontToBackWave<'_> {
    fn brightness(&self, settings: &Settings) -> u8 {
        settings.brightness()
    }

    fn render(
        &mut self, ws2812: &mut Ws2812<Spi<SPI0>>, timer: &mut Timer<TIMER0>, settings: &Settings,
    ) {
        animations::reset_data(self.data);

        let slice = VERTICAL_SLICES[self.position];

        for led in &slice {
            led.map(|l| {
                self.data.borrow_mut()[l as usize] =
                    brightness_correct(COLORS[settings.color_index()], self.brightness(settings));
            });
        }

        self.position = (self.position + 1) % VERTICAL_SLICES.len();

        ws2812.write(self.data.borrow().iter().copied()).unwrap();
        timer.delay_ms(settings.delay());
    }

    fn reset(&mut self) {
        animations::reset_data(self.data);
        self.position = 0;
    }
}
