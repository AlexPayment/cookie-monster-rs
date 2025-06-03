use crate::animations;
use crate::animations::{LedData, NUM_LEDS, Settings};
use embedded_hal::spi;
use embedded_hal_async::delay::DelayNs;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use smart_leds::RGB8;
use smart_leds::colors::{BLUE, DARK_RED, DARK_TURQUOISE, INDIGO, MIDNIGHT_BLUE, PURPLE, RED};
use smart_leds_trait::SmartLedsWrite;

const COLORS: [RGB8; 7] = [
    RED,
    DARK_RED,
    DARK_TURQUOISE,
    BLUE,
    MIDNIGHT_BLUE,
    PURPLE,
    INDIGO,
];
const NUM_STRANDS: usize = NUM_LEDS / 7;

pub struct MultiColorStrand<'a> {
    data: &'a LedData,
    prng: SmallRng,
    strands: [Strand; NUM_STRANDS],
}

impl<'a> MultiColorStrand<'a> {
    pub(crate) fn new(data: &'a LedData, random_seed: u64) -> Self {
        let mut prng = SmallRng::seed_from_u64(random_seed);

        let mut strands = [Strand::default(); NUM_STRANDS];

        for strand in &mut strands {
            strand.color_index = prng.random_range(0..COLORS.len()) as u8;
            strand.start = prng.random_range(0..NUM_LEDS) as u16;
            strand.end = prng.random_range(0..NUM_LEDS) as u16;
            while strand.start.abs_diff(strand.end) < (NUM_LEDS / 100) as u16 {
                strand.end = prng.random_range(0..NUM_LEDS) as u16;
            }
            strand.position = strand.start;
        }

        Self {
            data,
            prng,
            strands,
        }
    }

    pub(crate) async fn render(
        &mut self, ws2812: &mut impl SmartLedsWrite<Color = RGB8, Error = impl spi::Error>,
        delay: &mut impl DelayNs, settings: &Settings,
    ) {
        ws2812.write(self.data.borrow().iter().copied()).unwrap();
        delay.delay_ms(settings.delay()).await;
    }

    pub(crate) fn reset(&mut self) {
        animations::reset_data(self.data);

        for strand in &mut self.strands {
            strand.color_index = self.prng.random_range(0..COLORS.len()) as u8;
            strand.start = self.prng.random_range(0..NUM_LEDS) as u16;
            strand.end = self.prng.random_range(0..NUM_LEDS) as u16;
            while strand.start.abs_diff(strand.end) < 5 {
                strand.end = self.prng.random_range(0..NUM_LEDS) as u16;
            }
            strand.position = strand.start;
        }
    }

    pub(crate) fn update(&mut self, settings: &Settings) {
        animations::reset_data(self.data);

        let brightness = self.brightness(settings);

        for strand in &mut self.strands {
            strand.update();
            self.data.borrow_mut()[strand.position as usize] =
                animations::create_color_with_brightness(
                    COLORS[strand.color_index as usize],
                    brightness,
                );
        }
    }

    fn brightness(&self, settings: &Settings) -> f32 {
        settings.brightness()
    }
}

/// A single strand of LEDs with a color, start and end position, and current position.
///
/// The start could be greater or less than the end, which allows for the strand to move in either
/// direction.
///
/// Normally, all the fields would have been `usize`, but since the amount of memory available is
/// quite limited on most microcontrollers, it's better to use smaller types where possible. Which
/// allows us to have more strands.
#[derive(Clone, Copy, Default)]
struct Strand {
    color_index: u8,
    start: u16,
    end: u16,
    position: u16,
}

impl Strand {
    fn update(&mut self) {
        if self.start > self.end {
            self.position -= 1;
            if self.position == 0 {
                self.position = (NUM_LEDS - 1) as u16;
            }
            if self.position == self.end {
                self.position = self.start;
            }
        } else {
            self.position += 1;
            if self.position >= NUM_LEDS as u16 {
                self.position = 0;
            }
            if self.position == self.end {
                self.position = self.start;
            }
        }
    }
}
