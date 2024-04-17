use crate::animations;
use crate::animations::{Animation, Settings, UniColorSparkle, COLORS, NUM_LEDS, SHORTEST_DELAY};
use core::cell::RefCell;
use core::cmp;
use microbit::hal::gpio::{Output, Pin, PushPull};
use microbit::hal::prelude::_embedded_hal_blocking_delay_DelayMs;
use microbit::hal::spi::Spi;
use microbit::hal::timer::Periodic;
use microbit::hal::Timer;
use microbit::pac::{SPI0, TIMER0, TIMER1, TIMER2, TIMER3};
use rand::prelude::SmallRng;
use rand::{Rng, SeedableRng};
use rtt_target::rprintln;
use smart_leds::{SmartLedsWrite, RGB8};
use ws2812_spi::Ws2812;

impl<'a> UniColorSparkle<'a> {
    pub(crate) fn new(data: &'a RefCell<[RGB8; NUM_LEDS]>, random_seed: u64) -> Self {
        UniColorSparkle {
            data,
            prng: SmallRng::seed_from_u64(random_seed),
        }
    }

    fn change_data(&mut self, settings: &Settings) {
        animations::reset_data(self.data);

        // The amount of sparkles, up to 10% of the total number of LEDs
        let sparkle_amount = self.prng.gen_range(0..(NUM_LEDS / 10));
        for _ in 0..sparkle_amount {
            let index = self.prng.gen_range(0..NUM_LEDS);
            // Random brightness between 0% and the set brightness
            let brightness = self.prng.gen_range(0.0..settings.brightness);
            self.data.borrow_mut()[index] = animations::create_color_with_brightness(
                &COLORS[settings.color_index],
                &brightness,
            );
        }
    }
}

impl Animation for UniColorSparkle<'_> {
    fn render_spi(
        &mut self, ws2812: &mut Ws2812<Spi<SPI0>>, timer: &mut Timer<TIMER0>, settings: &Settings,
    ) {
        self.change_data(settings);

        let random_delay = self
            .prng
            .gen_range(SHORTEST_DELAY..cmp::max(settings.delay, SHORTEST_DELAY + 1));

        ws2812.write(self.data.borrow().iter().cloned()).unwrap();
        timer.delay_ms(random_delay);
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

        let random_delay = self
            .prng
            .gen_range(SHORTEST_DELAY..cmp::max(settings.delay, SHORTEST_DELAY + 1));

        let strip1_start_index = 0;
        let strip1_end_index = 96 * 1 - 1;

        let strip2_start_index = 96 * 1;
        let strip2_end_index = 96 * 2 - 1;

        let strip3_start_index = 96 * 2;
        let strip3_end_index = 96 * 3 - 1;

        let data = self.data.borrow();
        let strip1_data = &data[strip1_start_index..strip1_end_index];
        let strip2_data = &data[strip2_start_index..strip2_end_index];
        let strip3_data = &data[strip3_start_index..strip3_end_index];

        ws2812_strip1.write(strip1_data.iter().cloned()).unwrap();
        ws2812_strip2.write(strip2_data.iter().cloned()).unwrap();
        ws2812_strip3.write(strip3_data.iter().cloned()).unwrap();

        timer.delay_ms(random_delay);
    }
}
