use crate::animations;
use crate::animations::{Animation, MultiColorStrand, Settings, Strand, NUM_LEDS, NUM_STRANDS};
use core::cell::RefCell;
use embedded_hal::delay::DelayNs;
use microbit::pac::{SPI0, TIMER0};
use microbit::hal::spi::Spi;
use microbit::hal::Timer;
use rand::prelude::SmallRng;
use rand::{Rng, SeedableRng};
use smart_leds::colors::{BLUE, DARK_RED, DARK_TURQUOISE, INDIGO, MIDNIGHT_BLUE, PURPLE, RED};
use smart_leds::RGB8;
use smart_leds_trait::SmartLedsWrite;
use ws2812_spi::Ws2812;

const COLORS: [RGB8; 7] = [
    RED,
    DARK_RED,
    DARK_TURQUOISE,
    BLUE,
    MIDNIGHT_BLUE,
    PURPLE,
    INDIGO,
];

impl<'a> MultiColorStrand<'a> {
    pub fn new(data: &'a RefCell<[RGB8; NUM_LEDS]>, random_seed: u64) -> Self {
        let mut prng = SmallRng::seed_from_u64(random_seed);

        let mut strands = [Strand::default(); NUM_STRANDS];

        for strand in strands.iter_mut() {
            strand.color_index = prng.gen_range(0..COLORS.len());
            strand.start = prng.gen_range(0..NUM_LEDS);
            strand.end = prng.gen_range(0..NUM_LEDS);
            while strand.start.abs_diff(strand.end) < 5 {
                strand.end = prng.gen_range(0..NUM_LEDS);
            }
            strand.position = strand.start;
        }

        Self {
            data,
            prng,
            strands,
        }
    }
}

impl Animation for MultiColorStrand<'_> {
    fn brightness(&self, settings: &Settings) -> f32 {
        settings.brightness
    }

    fn render(
        &mut self, ws2812: &mut Ws2812<Spi<SPI0>>, timer: &mut Timer<TIMER0>, settings: &Settings,
    ) {
        animations::reset_data(self.data);

        let brightness = self.brightness(settings);

        for strand in self.strands.iter_mut() {
            update_strand(strand);
            self.data.borrow_mut()[strand.position] =
                animations::create_color_with_brightness(&COLORS[strand.color_index], brightness);
        }

        ws2812.write(self.data.borrow().iter().cloned()).unwrap();
        timer.delay_ms(settings.delay);
    }

    fn reset(&mut self) {
        animations::reset_data(self.data);

        for strand in self.strands.iter_mut() {
            strand.color_index = self.prng.gen_range(0..COLORS.len());
            strand.start = self.prng.gen_range(0..NUM_LEDS);
            strand.end = self.prng.gen_range(0..NUM_LEDS);
            while strand.start.abs_diff(strand.end) < 5 {
                strand.end = self.prng.gen_range(0..NUM_LEDS);
            }
            strand.position = strand.start;
        }
    }
}

fn update_strand(strand: &mut Strand) {
    if strand.start > strand.end {
        strand.position -= 1;
        if strand.position == 0 {
            strand.position = NUM_LEDS - 1;
        }
        if strand.position == strand.end {
            strand.position = strand.start;
        }
    } else {
        strand.position += 1;
        if strand.position >= NUM_LEDS {
            strand.position = 0;
        }
        if strand.position == strand.end {
            strand.position = strand.start;
        }
    }
}
