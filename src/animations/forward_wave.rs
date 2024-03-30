use crate::animations;
use crate::animations::{Animation, ForwardWave, Settings, COLORS, NUM_LEDS};
use core::cell::RefCell;
use microbit::hal::prelude::_embedded_hal_blocking_delay_DelayMs;
use microbit::hal::spi::Spi;
use microbit::hal::Timer;
use microbit::pac::{SPI0, TIMER0};
use smart_leds::{SmartLedsWrite, RGB8};
use ws2812_spi::Ws2812;

impl<'a> ForwardWave<'a> {
    pub(crate) fn new(data: &'a RefCell<[RGB8; NUM_LEDS]>) -> Self {
        ForwardWave { data, position: 0 }
    }
}

impl Animation for ForwardWave<'_> {
    fn render(
        &mut self, ws2812: &mut Ws2812<Spi<SPI0>>, timer: &mut Timer<TIMER0>, settings: &Settings,
    ) {
        animations::reset_data(self.data);


        // TODO: Make the wave size dynamic based on the number of LEDs
        let wave = [
            settings.brightness / 10.0,
            settings.brightness / 6.0,
            settings.brightness / 4.0,
            settings.brightness,
            settings.brightness / 10.0,
        ];

        // TODO: The wave shouldn't jump when it reaches the end
        for (i, item) in wave.iter().enumerate() {
            self.data.borrow_mut()[self.position + i] =
                animations::create_color_with_brightness(&COLORS[settings.color_index], item);
        }

        self.position += 1;
        if self.position >= NUM_LEDS - wave.len() {
            self.position = 0;
        }

        ws2812.write(self.data.borrow().iter().cloned()).unwrap();
        timer.delay_ms(settings.delay);
    }
}
