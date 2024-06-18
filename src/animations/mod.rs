use core::cell::RefCell;
use microbit::hal::spi::Spi;
use microbit::hal::Timer;
use microbit::pac::SPI0;
use rand::prelude::SmallRng;
use smart_leds::colors::{
    BLUE, DARK_GREEN, DARK_RED, DARK_TURQUOISE, GOLD, GREEN, INDIGO, MIDNIGHT_BLUE, PURPLE, RED,
    WHITE,
};
use smart_leds::RGB8;
use ws2812_spi::Ws2812;

pub(crate) mod carrousel;
pub(crate) mod double_carrousel;
pub(crate) mod forward_wave;
pub(crate) mod multi_color_fade_in;
pub(crate) mod multi_color_heartbeat;
pub(crate) mod multi_color_solid;
pub(crate) mod multi_color_solid_random;
pub(crate) mod multi_color_sparkle;
pub(crate) mod multi_color_strand;
pub(crate) mod uni_color_fade_in;
pub(crate) mod uni_color_heartbeat;
pub(crate) mod uni_color_solid;
pub(crate) mod uni_color_sparkle;

pub const NUM_COLORS: usize = 11;
pub const NUM_LEDS: usize = 96 * 10;
const NUM_STRANDS: usize = 250;
const SHORTEST_DELAY: u32 = 5;

pub(crate) trait Animation {
    fn render(
        &mut self, ws2812: &mut Ws2812<Spi<SPI0>>, timer: &mut Timer<microbit::pac::TIMER0>,
        settings: &Settings,
    );

    fn reset(&mut self);
}

pub(crate) const COLORS: [RGB8; NUM_COLORS] = [
    WHITE,
    RED,
    DARK_RED,
    GOLD,
    GREEN,
    DARK_GREEN,
    DARK_TURQUOISE,
    BLUE,
    MIDNIGHT_BLUE,
    PURPLE,
    INDIGO,
];

#[derive(Clone, Copy, Debug)]
pub struct Settings {
    pub brightness: f32,
    pub color_index: usize,
    pub delay: u32,
}

impl Settings {
    pub fn new(color_index: usize, brightness: f32, delay: u32) -> Self {
        Settings {
            brightness,
            color_index,
            delay,
        }
    }

    pub fn set_brightness(&mut self, brightness: f32) {
        self.brightness = brightness;
    }

    pub fn set_color_index(&mut self, color_index: usize) {
        self.color_index = color_index;
    }

    pub fn set_delay(&mut self, delay: u32) {
        self.delay = delay;
    }
}

pub(crate) struct Carrousel<'a> {
    color_index: usize,
    data: &'a RefCell<[RGB8; NUM_LEDS]>,
    position: usize,
    prng: SmallRng,
}

pub(crate) struct DoubleCarrousel<'a> {
    color_index_1: usize,
    color_index_2: usize,
    data: &'a RefCell<[RGB8; NUM_LEDS]>,
    position_1: usize,
    position_2: usize,
    prng: SmallRng,
}

pub(crate) struct ForwardWave<'a> {
    data: &'a RefCell<[RGB8; NUM_LEDS]>,
    position: usize,
    wrapped: bool,
}

pub(crate) struct MultiColorFadeIn<'a> {
    data: &'a RefCell<[RGB8; NUM_LEDS]>,
    ascending: bool,
    color_index: usize,
    prng: SmallRng,
    current_step: u8,
}

pub(crate) struct MultiColorHeartbeat<'a> {
    data: &'a RefCell<[RGB8; NUM_LEDS]>,
    color_index: usize,
    prng: SmallRng,
    current_step: u8,
    sequence: u8,
}

pub(crate) struct MultiColorSolid<'a> {
    data: &'a RefCell<[RGB8; NUM_LEDS]>,
}

pub(crate) struct MultiColorSolidRandom<'a> {
    data: &'a RefCell<[RGB8; NUM_LEDS]>,
    prng: SmallRng,
    rendered_data: [RGB8; NUM_LEDS],
}

pub(crate) struct MultiColorSparkle<'a> {
    data: &'a RefCell<[RGB8; NUM_LEDS]>,
    prng: SmallRng,
}

pub(crate) struct MultiColorStrand<'a> {
    data: &'a RefCell<[RGB8; NUM_LEDS]>,
    prng: SmallRng,
    strands: [Strand; NUM_STRANDS],
}

#[derive(Clone, Copy, Default)]
struct Strand {
    color_index: usize,
    start: usize,
    end: usize,
    position: usize,
}

pub(crate) struct UniColorFadeIn<'a> {
    data: &'a RefCell<[RGB8; NUM_LEDS]>,
    ascending: bool,
    current_step: u8,
}

pub(crate) struct UniColorHeartbeat<'a> {
    data: &'a RefCell<[RGB8; NUM_LEDS]>,
    current_step: u8,
    sequence: u8,
}

pub(crate) struct UniColorSolid<'a> {
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
