use core::cell::RefCell;
use embedded_time::duration::Milliseconds;
use embedded_time::fixed_point::FixedPoint;
use microbit::hal::prelude::_embedded_hal_blocking_delay_DelayMs;
use microbit::hal::{spi, Timer};
use rand::prelude::SmallRng;
use rand::{Rng, SeedableRng};
use smart_leds::{SmartLedsWrite, RGB8};
use ws2812_spi::Ws2812;

pub(crate) const NUM_LEDS: usize = 256;

pub(crate) trait Effect {
    fn render(
        &mut self, ws2812: &mut Ws2812<spi::Spi<microbit::pac::SPI0>>,
        delay: &mut Timer<microbit::pac::TIMER0>,
    );
}

pub(crate) struct Brightness {
    value: f32,
}

impl Brightness {
    pub(crate) const ONE: Brightness = Brightness { value: 0.01 };
    pub(crate) const FIVE: Brightness = Brightness { value: 0.05 };
    pub(crate) const TEN: Brightness = Brightness { value: 0.1 };
    pub(crate) const TWENTY_FIVE: Brightness = Brightness { value: 0.25 };
    pub(crate) const FIFTY: Brightness = Brightness { value: 0.5 };
    pub(crate) const SEVENTY_FIVE: Brightness = Brightness { value: 0.75 };
    pub(crate) const HUNDRED: Brightness = Brightness { value: 1.0 };
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

pub(crate) struct Settings {
    brightness: usize,
    brightnesses: [Brightness; 7],
    color: RGB8,
    delay: Milliseconds<u32>,
}

impl Settings {
    pub(crate) fn new(color: RGB8, speed: Speed) -> Self {
        Settings {
            brightness: 4,
            brightnesses: [
                Brightness::ONE,
                Brightness::FIVE,
                Brightness::TEN,
                Brightness::TWENTY_FIVE,
                Brightness::FIFTY,
                Brightness::SEVENTY_FIVE,
                Brightness::HUNDRED,
            ],
            color,
            delay: Milliseconds::<u32>(speed.value),
        }
    }

    pub(crate) fn cycle_brightness(&mut self) {
        if self.brightness >= self.brightnesses.len() - 1 {
            self.brightness = 0;
        } else {
            self.brightness += 1;
        }
    }

    fn get_brightness(&self) -> &Brightness {
        &self.brightnesses[self.brightness]
    }
}

pub(crate) struct ForwardWave<'a> {
    data: &'a RefCell<[RGB8; NUM_LEDS]>,
    position: usize,
    settings: &'a Settings,
}

impl<'a> ForwardWave<'a> {
    pub(crate) fn new(data: &'a RefCell<[RGB8; NUM_LEDS]>, settings: &'a Settings) -> Self {
        ForwardWave {
            data,
            position: 0,
            settings,
        }
    }
}

impl Effect for ForwardWave<'_> {
    fn render(
        &mut self, ws2812: &mut Ws2812<spi::Spi<microbit::pac::SPI0>>,
        delay: &mut Timer<microbit::pac::TIMER0>,
    ) {
        reset_data(self.data);

        let wave = [
            self.settings.get_brightness().value / 10.0,
            self.settings.get_brightness().value / 6.0,
            self.settings.get_brightness().value / 4.0,
            self.settings.get_brightness().value,
            self.settings.get_brightness().value / 10.0,
        ];
        for (i, item) in wave.iter().enumerate() {
            self.data.borrow_mut()[self.position + i] =
                create_color_with_brightness(&self.settings.color, item);
        }
        self.position += 1;
        if self.position >= NUM_LEDS - wave.len() {
            self.position = 0;
        }

        ws2812.write(self.data.borrow().iter().cloned()).unwrap();
        delay.delay_ms(self.settings.delay.integer() as u16);
    }
}

pub(crate) struct UniColorSparkle<'a> {
    data: &'a RefCell<[RGB8; NUM_LEDS]>,
    prng: SmallRng,
    settings: &'a Settings,
}

impl<'a> UniColorSparkle<'a> {
    pub(crate) fn new(
        data: &'a RefCell<[RGB8; NUM_LEDS]>, settings: &'a Settings, random_seed: u64,
    ) -> Self {
        UniColorSparkle {
            data,
            prng: SmallRng::seed_from_u64(random_seed),
            settings,
        }
    }
}

impl Effect for UniColorSparkle<'_> {
    fn render(
        &mut self, ws2812: &mut Ws2812<spi::Spi<microbit::pac::SPI0>>,
        delay: &mut Timer<microbit::pac::TIMER0>,
    ) {
        reset_data(self.data);

        // The amount of sparkles, up to 10% of the total number of LEDs
        let sparkle_amount = self.prng.gen_range(0..(NUM_LEDS / 10));
        for _ in 0..sparkle_amount {
            let index = self.prng.gen_range(0..NUM_LEDS);
            // Random brightness between 1% and the set brightness
            let brightness = self
                .prng
                .gen_range(Brightness::ONE.value..self.settings.get_brightness().value);
            self.data.borrow_mut()[index] =
                create_color_with_brightness(&self.settings.color, &brightness);
        }

        let random_delay = self.prng.gen_range(Speed::FASTEST.value..self.settings.delay.integer());

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
