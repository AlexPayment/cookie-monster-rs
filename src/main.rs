#![no_std]
#![no_main]

use core::cell::RefCell;
use cortex_m_rt::entry;
use embedded_time::duration::Milliseconds;
use embedded_time::fixed_point::FixedPoint;
use microbit::{hal, Peripherals};
use microbit::hal::{spi, Timer};
use microbit::hal::gpio::Level;
use microbit::hal::gpio::p0::Parts;
use microbit::hal::prelude::_embedded_hal_blocking_delay_DelayMs;
use panic_rtt_target as _;
use rand::{Rng, SeedableRng};
use rand::rngs::SmallRng;
use rtt_target::rtt_init_print;
use smart_leds::{RGB8, SmartLedsWrite};
use ws2812_spi::Ws2812;

mod controls;
mod cookie_monster;

const NUM_LEDS: usize = 256;

#[entry]
fn main() -> ! {
    rtt_init_print!();

    const DELAY: Milliseconds<u32> = Milliseconds::<u32>(500);

    // Setup all peripherals and the WS2812 device
    let peripherals = Peripherals::take().unwrap();
    let port0 = Parts::new(peripherals.P0);
    let sck = port0.p0_17.into_push_pull_output(Level::Low).degrade();
    // The SPI MOSI pin is pin 15 on the micro:bit.
    let mosi = port0.p0_13.into_push_pull_output(Level::Low).degrade();
    let miso = port0.p0_01.into_floating_input().degrade();
    let pins = spi::Pins {
        sck,
        miso: Some(miso),
        mosi: Some(mosi),
    };
    let spi = spi::Spi::new(peripherals.SPI0, pins, spi::Frequency::M4, spi::MODE_0);
    let mut ws2812 = Ws2812::new(spi);
    let mut delay = Timer::new(peripherals.TIMER0);

    // Setup Pseudo Random Number Generator
    let mut rng = hal::Rng::new(peripherals.RNG);

    // let cookie_monster = CookieMonster::new(board.display_pins, board.TIMER0);
    // init_buttons(board.GPIOTE, board.buttons, cookie_monster);

    let data = RefCell::new([RGB8::default(); NUM_LEDS]);
    let settings = Settings::new(RGB8::new(0x00, 0x00, 0xff), Brightness::HUNDRED, Speed::FAST);

    let mut uni_color_sparkle = UniColorSparkle::new(&data, &settings, rng.random_u64());
    let mut forward_wave = ForwardWave::new(&data, &settings);

    let mut effect: [&mut dyn Effect; 2] = [&mut uni_color_sparkle, &mut forward_wave];

    loop {
        effect[0].render(&mut ws2812, &mut delay);
    }
}

fn reset_data(data: &RefCell<[RGB8; NUM_LEDS]>) {
    let mut data = data.borrow_mut();
    for i in 0..NUM_LEDS {
        data[i] = RGB8::default();
    }
}

struct Brightness {
    value: f32,
}

impl Brightness {
    const ONE: Brightness = Brightness { value: 0.01 };
    const FIVE: Brightness = Brightness { value: 0.05 };
    const TEN: Brightness = Brightness { value: 0.1 };
    const TWENTY_FIVE: Brightness = Brightness { value: 0.25 };
    const FIFTY: Brightness = Brightness { value: 0.5 };
    const SEVENTY_FIVE: Brightness = Brightness { value: 0.75 };
    const HUNDRED: Brightness = Brightness { value: 1.0 };
}

struct Settings {
    brightness: Brightness,
    color: RGB8,
    delay: Milliseconds<u32>,
}

impl Settings {
    fn new(color: RGB8, brightness: Brightness, speed: Speed) -> Self {
        Settings {
            brightness,
            color,
            delay: Milliseconds::<u32>(speed.value),
        }
    }
}

struct Speed {
    value: u32,
}

impl Speed {
    const SLOWEST: Speed = Speed { value: 2000 };
    const SLOW: Speed = Speed { value: 1000 };
    const MEDIUM: Speed = Speed { value: 500 };
    const MEDIUM_FAST: Speed = Speed { value: 200 };
    const FAST: Speed = Speed { value: 100 };
    const FASTER: Speed = Speed { value: 50 };
    const FASTEST: Speed = Speed { value: 10 };
}

trait Effect {
    fn render(&mut self, ws2812: &mut Ws2812<spi::Spi<microbit::pac::SPI0>>, delay: &mut Timer<microbit::pac::TIMER0>);
}

struct ForwardWave<'a> {
    data: &'a RefCell<[RGB8; NUM_LEDS]>,
    position: usize,
    settings: &'a Settings,
}

impl <'a> ForwardWave<'a> {
    fn new(data: &'a RefCell<[RGB8; NUM_LEDS]>, settings: &'a Settings) -> Self {
        ForwardWave {
            data,
            position: 0,
            settings,
        }
    }
}

impl Effect for ForwardWave<'_> {
    fn render(&mut self, ws2812: &mut Ws2812<spi::Spi<microbit::pac::SPI0>>, delay: &mut Timer<microbit::pac::TIMER0>) {
        reset_data(self.data);

        let wave = [self.settings.brightness.value / 10.0, self.settings.brightness.value / 6.0 , self.settings.brightness.value / 4.0, self.settings.brightness.value, self.settings.brightness.value / 10.0];
        for i in 0..wave.len() {
            self.data.borrow_mut()[self.position + i] = RGB8::new((self.settings.color.r as f32 * wave[i]) as u8, (self.settings.color.g as f32 * wave[i]) as u8, (self.settings.color.b as f32 * wave[i]) as u8);
        }
        self.position += 1;
        if self.position >= NUM_LEDS - wave.len() {
            self.position = 0;
        }

        ws2812.write(self.data.borrow().iter().cloned()).unwrap();
        delay.delay_ms(self.settings.delay.integer() as u16);
    }
}

struct UniColorSparkle<'a> {
    data: &'a RefCell<[RGB8; NUM_LEDS]>,
    prng: SmallRng,
    settings: &'a Settings,
}

impl <'a> UniColorSparkle<'a> {
    fn new(data: &'a RefCell<[RGB8; NUM_LEDS]>, settings: &'a Settings, random_seed: u64) -> Self {
        UniColorSparkle {
            data,
            prng : SmallRng::seed_from_u64(random_seed),
            settings,
        }
    }
}

impl Effect for UniColorSparkle<'_> {
    fn render(&mut self, ws2812: &mut Ws2812<spi::Spi<microbit::pac::SPI0>>, delay: &mut Timer<microbit::pac::TIMER0>) {
        reset_data(self.data);

        let sparkle_amount = self.prng.gen_range(0..(NUM_LEDS / 10));
        for _ in 0..sparkle_amount {
            let index = self.prng.gen_range(0..NUM_LEDS);
            let brightness = self.prng.gen_range(Brightness::ONE.value..self.settings.brightness.value);
            self.data.borrow_mut()[index] = RGB8::new((self.settings.color.r as f32 * brightness) as u8, (self.settings.color.g as f32 * brightness) as u8, (self.settings.color.b as f32 * brightness) as u8);
        }

        ws2812.write(self.data.borrow().iter().cloned()).unwrap();
        delay.delay_ms(self.settings.delay.integer() as u16);
    }
}