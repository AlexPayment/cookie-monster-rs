use crate::animations;
use crate::animations::{LEDS_TOTAL, LedData};
use rand::rngs::SmallRng;
use rand::{RngExt, SeedableRng};
use smart_leds::RGB8;
use smart_leds::colors::{BLUE, DARK_RED, DARK_TURQUOISE, INDIGO, MIDNIGHT_BLUE, PURPLE, RED};

const COLORS: [RGB8; 7] = [
    RED,
    DARK_RED,
    DARK_TURQUOISE,
    BLUE,
    MIDNIGHT_BLUE,
    PURPLE,
    INDIGO,
];
const NUM_STRANDS: usize = LEDS_TOTAL / 7;

pub struct MultiColorStrand {
    strands: [Strand; NUM_STRANDS],
}

impl MultiColorStrand {
    pub(crate) const BRIGHTNESS_DAMPING_FACTOR: f32 = 1.0;

    pub(crate) fn new(random_seed: u64) -> Self {
        let mut prng = SmallRng::seed_from_u64(random_seed);

        let mut strands = [Strand::default(); NUM_STRANDS];

        for strand in &mut strands {
            strand.color_index = prng.random_range(0..COLORS.len()) as u8;
            strand.start = prng.random_range(0..LEDS_TOTAL) as u16;
            strand.end = prng.random_range(0..LEDS_TOTAL) as u16;
            // Let's make sure the strands are not too short.
            while strand.start.abs_diff(strand.end) < (LEDS_TOTAL / 100) as u16 {
                strand.end = prng.random_range(0..LEDS_TOTAL) as u16;
            }
            strand.position = strand.start;
        }

        Self { strands }
    }

    pub(crate) fn update(&mut self, data: &mut LedData) {
        animations::reset_data(data);

        for strand in &mut self.strands {
            strand.update();
            data[usize::from(strand.position)] = COLORS[usize::from(strand.color_index)];
        }
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
                self.position = (LEDS_TOTAL - 1) as u16;
            }
            if self.position == self.end {
                self.position = self.start;
            }
        } else {
            self.position += 1;
            if self.position >= LEDS_TOTAL as u16 {
                self.position = 0;
            }
            if self.position == self.end {
                self.position = self.start;
            }
        }
    }
}
