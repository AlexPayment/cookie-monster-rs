use crate::animations;
use crate::animations::{Animation, ForwardWave, Settings, COLORS, NUM_LEDS};
use core::cell::RefCell;
use microbit::hal::gpio::{Output, Pin, PushPull};
use microbit::hal::prelude::_embedded_hal_blocking_delay_DelayMs;
use microbit::hal::spi::Spi;
use microbit::hal::timer::Periodic;
use microbit::hal::Timer;
use microbit::pac::{SPI0, TIMER0, TIMER1, TIMER2, TIMER3};
use smart_leds::{SmartLedsWrite, RGB8};
use ws2812_spi::Ws2812;

impl<'a> ForwardWave<'a> {
    pub(crate) fn new(data: &'a RefCell<[RGB8; NUM_LEDS]>) -> Self {
        ForwardWave { data, position: 0 }
    }

    fn change_data(&mut self, settings: &Settings) {
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
    }
}

impl Animation for ForwardWave<'_> {
    fn render_spi(
        &mut self, ws2812: &mut Ws2812<Spi<SPI0>>, timer: &mut Timer<TIMER0>, settings: &Settings,
    ) {
        self.change_data(settings);

        ws2812.write(self.data.borrow().iter().cloned()).unwrap();
        timer.delay_ms(settings.delay);
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
        self.change_data(settings);

        let strip1_start_index = 0;
        let strip1_end_index = 96 * 3 - 1;

        let strip2_start_index = 96 * 3;
        let strip2_end_index = 96 * 5 - 1;

        let strip3_start_index = 96 * 5;
        let strip3_end_index = 96 * 7 - 1;

        let data = self.data.borrow();
        let strip1_data = &data[strip1_start_index..strip1_end_index];
        let strip2_data = &data[strip2_start_index..strip2_end_index];
        let strip3_data = &data[strip3_start_index..strip3_end_index];

        ws2812_strip1.write(strip1_data.iter().cloned()).unwrap();
        ws2812_strip2.write(strip2_data.iter().cloned()).unwrap();
        ws2812_strip3.write(strip3_data.iter().cloned()).unwrap();

        timer.delay_ms(settings.delay);
    }
}
