use crate::animations;
use crate::animations::{LedData, NUM_LEDS, Settings};
use embedded_hal::spi::Error as SpiError;
use embedded_hal_async::delay::DelayNs;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use smart_leds::RGB8;
use smart_leds_trait::SmartLedsWrite;

pub struct MultiColorSolidRandom<'a> {
    data: &'a LedData,
    prng: SmallRng,
    rendered_data: [RGB8; NUM_LEDS],
}

impl<'a> MultiColorSolidRandom<'a> {
    pub(crate) fn new(data: &'a LedData, random_seed: u64) -> Self {
        let mut animation = Self {
            data,
            prng: SmallRng::seed_from_u64(random_seed),
            rendered_data: [RGB8::default(); NUM_LEDS],
        };

        for i in 0..NUM_LEDS {
            let random_color = RGB8::new(
                animation.prng.random_range(0..255),
                animation.prng.random_range(0..255),
                animation.prng.random_range(0..255),
            );
            animation.rendered_data[i] = random_color;
        }

        animation
    }

    pub(crate) async fn render(
        &mut self, ws2812: &mut impl SmartLedsWrite<Color = RGB8, Error = impl SpiError>,
        delay: &mut impl DelayNs,
    ) {
        ws2812.write(self.data.borrow().iter().copied()).unwrap();
        // Delay from the settings doesn't really matter for the solid animations. So just using a
        // 1-second delay.
        delay.delay_ms(1_000u32).await;
    }

    pub(crate) fn reset(&mut self) {
        animations::reset_data(self.data);
    }

    pub(crate) fn update(&mut self, settings: &Settings) {
        for i in 0..NUM_LEDS {
            self.data.borrow_mut()[i] = animations::create_color_with_brightness(
                self.rendered_data[i],
                self.brightness(settings),
            );
        }
    }

    fn brightness(&self, settings: &Settings) -> f32 {
        settings.brightness() * 0.2
    }
}
