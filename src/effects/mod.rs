use core::cell::RefCell;
use cortex_m::interrupt::{free, Mutex};
use embedded_time::duration::Milliseconds;
use embedded_time::fixed_point::FixedPoint;
use microbit::hal::prelude::_embedded_hal_blocking_delay_DelayMs;
use microbit::hal::spi::Spi;
use microbit::hal::Timer;
use microbit::pac::{SPI0, TIMER0};
use rand::prelude::SmallRng;
use rand::{Rng, SeedableRng};
use smart_leds::{SmartLedsWrite, RGB8};
use ws2812_spi::Ws2812;

pub(crate) const NUM_LEDS: usize = 256;

pub static SETTINGS: Mutex<RefCell<Option<Settings>>> = Mutex::new(RefCell::new(None));

pub(crate) trait Effect {
    fn render(
        &mut self, ws2812: &mut Ws2812<Spi<SPI0>>, delay: &mut Timer<microbit::pac::TIMER0>,
        brightness: f32,
    );
}

pub(crate) struct Speed {
    value: u32,
}

impl Speed {
    pub(crate) const SLOWEST: Speed = Speed { value: 2000 };
    pub(crate) const SLOW: Speed = Speed { value: 1000 };
    pub(crate) const MEDIUM: Speed = Speed { value: 500 };
    pub(crate) const MEDIUM_FAST: Speed = Speed { value: 200 };
    pub(crate) const FAST: Speed = Speed { value: 100 };
    pub(crate) const FASTER: Speed = Speed { value: 50 };
    pub(crate) const FASTEST: Speed = Speed { value: 10 };
}

#[derive(Clone, Copy)]
pub(crate) struct Settings {
    color: RGB8,
    delay: Milliseconds<u32>,
}

impl Settings {
    pub(crate) fn new(color: RGB8, speed: Speed) -> Self {
        Settings {
            color,
            delay: Milliseconds::<u32>(speed.value),
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

impl Effect for ForwardWave<'_> {
    fn render(
        &mut self, ws2812: &mut Ws2812<Spi<SPI0>>, delay: &mut Timer<TIMER0>, brightness: f32,
    ) {
        reset_data(self.data);

        let mut color = RGB8::new(0x00, 0x00, 0xff);
        let mut delay_settings = Speed::MEDIUM.value;

        free(|cs| {
            if let Some(settings) = SETTINGS.borrow(cs).borrow().as_ref() {
                color = settings.color;
                delay_settings = settings.delay.integer();
            }
        });

        let wave = [
            brightness / 10.0,
            brightness / 6.0,
            brightness / 4.0,
            brightness,
            brightness / 10.0,
        ];
        for (i, item) in wave.iter().enumerate() {
            self.data.borrow_mut()[self.position + i] = create_color_with_brightness(&color, item);
        }
        self.position += 1;
        if self.position >= NUM_LEDS - wave.len() {
            self.position = 0;
        }

        ws2812.write(self.data.borrow().iter().cloned()).unwrap();
        delay.delay_ms(delay_settings as u16);
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

impl Effect for UniColorSparkle<'_> {
    fn render(
        &mut self, ws2812: &mut Ws2812<Spi<SPI0>>, delay: &mut Timer<TIMER0>, brightness: f32,
    ) {
        reset_data(self.data);

        let mut color = RGB8::new(0x00, 0x00, 0xff);
        let mut delay_settings = Speed::MEDIUM.value;

        free(|cs| {
            if let Some(settings) = SETTINGS.borrow(cs).borrow().as_ref() {
                color = settings.color;
                delay_settings = settings.delay.integer();
            }
        });

        // The amount of sparkles, up to 10% of the total number of LEDs
        let sparkle_amount = self.prng.gen_range(0..(NUM_LEDS / 10));
        for _ in 0..sparkle_amount {
            let index = self.prng.gen_range(0..NUM_LEDS);
            // Random brightness between 1% and the set brightness
            let brightness = self.prng.gen_range(0.0..brightness);
            self.data.borrow_mut()[index] = create_color_with_brightness(&color, &brightness);
        }

        let random_delay = self.prng.gen_range(Speed::FASTEST.value..delay_settings);

        ws2812.write(self.data.borrow().iter().cloned()).unwrap();
        delay.delay_ms(random_delay as u16);
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
