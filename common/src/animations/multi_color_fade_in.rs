use crate::animations;
use crate::animations::{COLORS, LedData, NUM_COLORS, NUM_LEDS, Settings};
use embedded_hal::spi::Error as SpiError;
use embedded_hal_async::delay::DelayNs;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use smart_leds::RGB8;
use smart_leds_trait::SmartLedsWrite;

const STEP: u8 = 23;

pub struct MultiColorFadeIn<'a> {
    data: &'a LedData,
    ascending: bool,
    color_index: usize,
    prng: SmallRng,
    current_step: u8,
}

impl<'a> MultiColorFadeIn<'a> {
    pub(crate) fn new(data: &'a LedData, random_seed: u64) -> Self {
        let mut prng = SmallRng::seed_from_u64(random_seed);
        Self {
            data,
            ascending: true,
            color_index: prng.random_range(0..NUM_COLORS),
            prng,
            current_step: 0,
        }
    }

    pub(crate) async fn render(
        &mut self, ws2812: &mut impl SmartLedsWrite<Color = RGB8, Error = impl SpiError>,
        delay: &mut impl DelayNs, settings: &Settings,
    ) {
        ws2812.write(self.data.borrow().iter().copied()).unwrap();
        delay.delay_ms(settings.delay()).await;
    }

    pub(crate) fn reset(&mut self) {
        animations::reset_data(self.data);
        self.ascending = true;
        self.current_step = 0;
    }

    pub(crate) fn update(&mut self, settings: &Settings) {
        let brightness =
            (self.brightness(settings) / f32::from(STEP)) * f32::from(self.current_step);
        let color = animations::create_color_with_brightness(COLORS[self.color_index], brightness);
        for i in 0..NUM_LEDS {
            self.data.borrow_mut()[i] = color;
        }
        if self.ascending {
            self.current_step += 1;
            if self.current_step >= STEP {
                self.ascending = false;
            }
        } else {
            self.current_step -= 1;
            if self.current_step == 1 {
                self.color_index = self.prng.random_range(0..NUM_COLORS);
                self.ascending = true;
            }
        }
    }

    fn brightness(&self, settings: &Settings) -> f32 {
        settings.brightness() * 0.05
    }
}
