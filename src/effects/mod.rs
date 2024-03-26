use core::cell::RefCell;
use core::cmp;
use microbit::hal::prelude::_embedded_hal_blocking_delay_DelayMs;
use microbit::hal::spi::Spi;
use microbit::hal::Timer;
use microbit::pac::{SPI0, TIMER0};
use rand::prelude::SmallRng;
use rand::{Rng, SeedableRng};
use smart_leds::colors::{
    AQUA, BLUE, FUCHSIA, GREEN, LIME, MAROON, NAVY, OLIVE, PURPLE, RED, TEAL, WHITE, YELLOW,
};
use smart_leds::{SmartLedsWrite, RGB8};
use ws2812_spi::Ws2812;

pub(crate) const NUM_COLORS: usize = 13;
pub(crate) const NUM_LEDS: usize = 256;
const SHORTEST_DELAY: u32 = 5;

pub(crate) trait Animation {
    fn render(
        &mut self, ws2812: &mut Ws2812<Spi<SPI0>>, timer: &mut Timer<microbit::pac::TIMER0>,
        settings: &Settings,
    );
}

pub(crate) const COLORS: [RGB8; NUM_COLORS] = [
    WHITE, RED, MAROON, YELLOW, OLIVE, LIME, GREEN, AQUA, TEAL, BLUE, NAVY, FUCHSIA, PURPLE,
];

#[derive(Clone, Copy, Debug)]
pub(crate) struct Settings {
    pub(crate) brightness: f32,
    pub(crate) color_index: usize,
    pub(crate) delay: u32,
}

impl Settings {
    pub(crate) fn new(color_index: usize, brightness: f32, delay: u32) -> Self {
        Settings {
            brightness,
            color_index,
            delay,
        }
    }
}

pub(crate) struct ForwardWave<'a> {
    data: &'a RefCell<[RGB8; NUM_LEDS]>,
    position: usize,
}

impl<'a> ForwardWave<'a> {
    pub(crate) fn new(data: &'a RefCell<[RGB8; NUM_LEDS]>) -> Self {
        ForwardWave { data, position: 0 }
    }
}

impl Animation for ForwardWave<'_> {
    fn render(
        &mut self, ws2812: &mut Ws2812<Spi<SPI0>>, timer: &mut Timer<TIMER0>, settings: &Settings,
    ) {
        reset_data(self.data);

        let wave = [
            settings.brightness / 10.0,
            settings.brightness / 6.0,
            settings.brightness / 4.0,
            settings.brightness,
            settings.brightness / 10.0,
        ];
        for (i, item) in wave.iter().enumerate() {
            self.data.borrow_mut()[self.position + i] =
                create_color_with_brightness(&COLORS[settings.color_index], item);
        }
        self.position += 1;
        if self.position >= NUM_LEDS - wave.len() {
            self.position = 0;
        }

        ws2812.write(self.data.borrow().iter().cloned()).unwrap();
        timer.delay_ms(settings.delay as u16);
    }
}

pub(crate) struct MultiColorSparkle<'a> {
    data: &'a RefCell<[RGB8; NUM_LEDS]>,
    prng: SmallRng,
}

impl<'a> MultiColorSparkle<'a> {
    pub(crate) fn new(data: &'a RefCell<[RGB8; NUM_LEDS]>, random_seed: u64) -> Self {
        MultiColorSparkle {
            data,
            prng: SmallRng::seed_from_u64(random_seed),
        }
    }
}

impl Animation for MultiColorSparkle<'_> {
    fn render(
        &mut self, ws2812: &mut Ws2812<Spi<SPI0>>, timer: &mut Timer<TIMER0>, settings: &Settings,
    ) {
        reset_data(self.data);

        // The amount of sparkles, up to 10% of the total number of LEDs
        let sparkle_amount = self.prng.gen_range(0..(NUM_LEDS / 10));
        for _ in 0..sparkle_amount {
            let index = self.prng.gen_range(0..NUM_LEDS);
            // Random brightness between 0% and the set brightness
            let brightness = self.prng.gen_range(0.0..settings.brightness);
            let random_color = RGB8::new(
                self.prng.gen_range(0..255),
                self.prng.gen_range(0..255),
                self.prng.gen_range(0..255),
            );
            self.data.borrow_mut()[index] =
                create_color_with_brightness(&random_color, &brightness);
        }

        let random_delay = self
            .prng
            .gen_range(SHORTEST_DELAY..cmp::max(settings.delay, SHORTEST_DELAY + 1));

        ws2812.write(self.data.borrow().iter().cloned()).unwrap();
        timer.delay_ms(random_delay as u16);
    }
}

pub(crate) struct UniColorSparkle<'a> {
    data: &'a RefCell<[RGB8; NUM_LEDS]>,
    prng: SmallRng,
}

impl<'a> UniColorSparkle<'a> {
    pub(crate) fn new(data: &'a RefCell<[RGB8; NUM_LEDS]>, random_seed: u64) -> Self {
        UniColorSparkle {
            data,
            prng: SmallRng::seed_from_u64(random_seed),
        }
    }
}

impl Animation for UniColorSparkle<'_> {
    fn render(
        &mut self, ws2812: &mut Ws2812<Spi<SPI0>>, timer: &mut Timer<TIMER0>, settings: &Settings,
    ) {
        reset_data(self.data);

        // The amount of sparkles, up to 10% of the total number of LEDs
        let sparkle_amount = self.prng.gen_range(0..(NUM_LEDS / 10));
        for _ in 0..sparkle_amount {
            let index = self.prng.gen_range(0..NUM_LEDS);
            // Random brightness between 0% and the set brightness
            let brightness = self.prng.gen_range(0.0..settings.brightness);
            self.data.borrow_mut()[index] =
                create_color_with_brightness(&COLORS[settings.color_index], &brightness);
        }

        let random_delay = self
            .prng
            .gen_range(SHORTEST_DELAY..cmp::max(settings.delay, SHORTEST_DELAY + 1));

        ws2812.write(self.data.borrow().iter().cloned()).unwrap();
        timer.delay_ms(random_delay as u16);
    }
}

fn create_color_with_brightness(color: &RGB8, brightness: &f32) -> RGB8 {
    RGB8::new(
        (color.r as f32 * brightness) as u8,
        (color.g as f32 * brightness) as u8,
        (color.b as f32 * brightness) as u8,
    )
}

fn reset_data(data: &RefCell<[RGB8; NUM_LEDS]>) {
    let mut data = data.borrow_mut();
    for i in 0..NUM_LEDS {
        data[i] = RGB8::default();
    }
}
