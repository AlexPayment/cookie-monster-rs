use cookie_monster_common::animations::{LedData, NUM_LEDS, Settings};
use microbit::hal::Timer;
use microbit::hal::spi::Spi;
use microbit::pac::SPI0;
use rand::rngs::SmallRng;
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
pub(crate) mod uni_color_front_to_back_wave;
pub(crate) mod uni_color_heartbeat;
pub(crate) mod uni_color_solid;
pub(crate) mod uni_color_sparkle;

pub(crate) trait Animation {
    fn brightness(&self, settings: &Settings) -> u8;

    fn render(
        &mut self, ws2812: &mut Ws2812<Spi<SPI0>>, timer: &mut Timer<microbit::pac::TIMER0>,
        settings: &Settings,
    );

    fn reset(&mut self);
}

const NUM_STRANDS: usize = NUM_LEDS / 7;

pub(crate) struct Carrousel<'a> {
    color_index: usize,
    data: &'a LedData,
    position: usize,
    prng: SmallRng,
}

pub(crate) struct DoubleCarrousel<'a> {
    color_index_1: usize,
    color_index_2: usize,
    data: &'a LedData,
    position_1: usize,
    position_2: usize,
    prng: SmallRng,
}

pub(crate) struct ForwardWave<'a> {
    data: &'a LedData,
    position: usize,
    wrapped: bool,
}

pub(crate) struct MultiColorFadeIn<'a> {
    data: &'a LedData,
    ascending: bool,
    color_index: usize,
    prng: SmallRng,
    current_step: u8,
}

pub(crate) struct MultiColorHeartbeat<'a> {
    data: &'a LedData,
    color_index: usize,
    prng: SmallRng,
    current_step: u8,
    sequence: u8,
}

pub(crate) struct MultiColorSolid<'a> {
    data: &'a LedData,
}

pub(crate) struct MultiColorSolidRandom<'a> {
    data: &'a LedData,
    prng: SmallRng,
    rendered_data: [RGB8; NUM_LEDS],
}

pub(crate) struct MultiColorSparkle<'a> {
    data: &'a LedData,
    prng: SmallRng,
}

pub(crate) struct MultiColorStrand<'a> {
    data: &'a LedData,
    prng: SmallRng,
    strands: [Strand; NUM_STRANDS],
}

#[derive(Clone, Copy, Default)]
struct Strand {
    color_index: u8,
    start: u16,
    end: u16,
    position: u16,
}

pub(crate) struct UniColorFadeIn<'a> {
    data: &'a LedData,
    ascending: bool,
    current_step: u8,
}

pub(crate) struct UniColorFrontToBackWave<'a> {
    data: &'a LedData,
    position: usize,
}

pub(crate) struct UniColorHeartbeat<'a> {
    data: &'a LedData,
    current_step: u8,
    sequence: u8,
}

pub(crate) struct UniColorSolid<'a> {
    data: &'a LedData,
}

pub(crate) struct UniColorSparkle<'a> {
    data: &'a LedData,
    prng: SmallRng,
}
