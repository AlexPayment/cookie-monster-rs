use crate::animations::{
    COLORS, LEDS_SECTION_1_RANGE, LEDS_SECTION_2_RANGE, LedData, Settings, VERTICAL_SLICES,
};
use core::fmt::Debug;
use embassy_futures::join::join;
use embedded_hal_async::delay::DelayNs;
use rand::prelude::SmallRng;
use rand::{RngExt, SeedableRng};
use smart_leds::{RGB8, brightness, gamma};
use smart_leds_trait::SmartLedsWrite;

const BRIGHTNESS_DAMPING_FACTOR: f32 = 0.2;
const SLICES_EVEN: [usize; 8] = [0, 2, 4, 6, 8, 10, 12, 14];
const SLICES_ODD: [usize; 8] = [1, 3, 5, 7, 9, 11, 13, 15];

pub struct MultiColorFrontToBackWave {
    color_index: usize,
    even: bool,
    position: usize,
    prng: SmallRng,
}

impl MultiColorFrontToBackWave {
    pub(crate) fn new(random_seed: u64) -> Self {
        let mut prng = SmallRng::seed_from_u64(random_seed);

        let color_index = prng.random_range(0..COLORS.len());
        Self {
            color_index,
            even: true,
            position: 0,
            prng,
        }
    }

    pub(crate) async fn render(
        &mut self, data: &LedData,
        leds_section_1: &mut impl SmartLedsWrite<Color = RGB8, Error = impl Debug>,
        leds_section_2: &mut impl SmartLedsWrite<Color = RGB8, Error = impl Debug>,
        delay: &mut impl DelayNs, settings: &Settings,
    ) {
        let leds_section_1_future = async {
            leds_section_1
                .write(brightness(
                    gamma(data[LEDS_SECTION_1_RANGE].iter().copied()),
                    settings.brightness_damped(BRIGHTNESS_DAMPING_FACTOR),
                ))
                .unwrap();
        };

        let leds_section_2_future = async {
            leds_section_2
                .write(brightness(
                    gamma(data[LEDS_SECTION_2_RANGE].iter().copied()),
                    settings.brightness_damped(BRIGHTNESS_DAMPING_FACTOR),
                ))
                .unwrap();
        };

        join(leds_section_1_future, leds_section_2_future).await;

        delay.delay_ms(settings.delay()).await;
    }

    pub(crate) fn update(&mut self, data: &mut LedData) {
        let slice_index = if self.even {
            SLICES_EVEN[self.position]
        } else {
            SLICES_ODD[self.position]
        };
        let slice = &VERTICAL_SLICES[slice_index];

        for led in slice {
            led.map(|l| {
                data[usize::from(l)] = COLORS[self.color_index];
            });
        }

        // When we wrap back to the beginning, pick a new color and flip the even/odd flag
        if self.position == SLICES_EVEN.len() - 1 {
            self.color_index = self.pick_new_color();
            self.even = !self.even;
        }

        self.position = (self.position + 1) % SLICES_EVEN.len();
    }

    fn pick_new_color(&mut self) -> usize {
        let mut new_color = self.prng.random_range(0..COLORS.len());
        while self.color_index == new_color {
            new_color = self.prng.random_range(0..COLORS.len());
        }
        new_color
    }
}
