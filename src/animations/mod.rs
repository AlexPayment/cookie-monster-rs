use core::cell::RefCell;
use microbit::hal::spi::Spi;
use microbit::hal::Timer;
use microbit::pac::SPI0;
use rand::prelude::SmallRng;
use smart_leds::colors::{
    AQUA, BLUE, FUCHSIA, GREEN, LIME, MAROON, NAVY, OLIVE, PURPLE, RED, TEAL, WHITE, YELLOW,
};
use smart_leds::RGB8;
use ws2812_spi::Ws2812;

pub(crate) mod forward_wave;
pub(crate) mod multi_color_fade_in;
pub(crate) mod multi_color_hearthbeat;
pub(crate) mod multi_color_sparkle;
pub(crate) mod uni_color_fade_in;
pub(crate) mod uni_color_hearthbeat;
pub(crate) mod uni_color_solid;
pub(crate) mod uni_color_sparkle;

pub(crate) const NUM_COLORS: usize = 13;
pub(crate) const NUM_LEDS: usize = 256;
const SHORTEST_DELAY: u16 = 5;

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
    pub(crate) delay: u16,
}

impl Settings {
    pub(crate) fn new(color_index: usize, brightness: f32, delay: u16) -> Self {
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

pub(crate) struct MultiColorFadeIn<'a> {
    data: &'a RefCell<[RGB8; NUM_LEDS]>,
    prng: SmallRng,
}

pub(crate) struct MultiColorHeartbeat<'a> {
    data: &'a RefCell<[RGB8; NUM_LEDS]>,
    prng: SmallRng,
}

pub(crate) struct MultiColorSparkle<'a> {
    data: &'a RefCell<[RGB8; NUM_LEDS]>,
    prng: SmallRng,
}

pub(crate) struct UniColorSolid<'a> {
    data: &'a RefCell<[RGB8; NUM_LEDS]>,
}

pub(crate) struct UniColorFadeIn<'a> {
    data: &'a RefCell<[RGB8; NUM_LEDS]>,
}

pub(crate) struct UniColorHeartbeat<'a> {
    data: &'a RefCell<[RGB8; NUM_LEDS]>,
}

pub(crate) struct UniColorSparkle<'a> {
    data: &'a RefCell<[RGB8; NUM_LEDS]>,
    prng: SmallRng,
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
