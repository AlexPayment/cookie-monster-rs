#![no_std]
#![no_main]

use cortex_m_rt::entry;
use embedded_time::duration::Milliseconds;
use embedded_time::fixed_point::FixedPoint;
use microbit::{hal, Peripherals};
use microbit::hal::{spi, Timer};
use microbit::hal::gpio::Level;
use microbit::hal::gpio::p0::Parts;
use microbit::hal::prelude::_embedded_hal_blocking_delay_DelayMs;
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

    let mut uni_color_sparkle = UniColorSparkle::new(RGB8::new(0x00, 0x00, 0xff), Brightness::HUNDRED, Speed::MEDIUM_FAST, rng.random_u64());
    let mut forward_wave = ForwardWave::new(RGB8::new(0x00, 0x00, 0xff), Brightness::TWENTY_FIVE, Speed::FAST);

    loop {
        // uni_color_sparkle.render(&mut ws2812, &mut delay);
        forward_wave.render(&mut ws2812, &mut delay);
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

struct ForwardWave {
    brightness_max: Brightness,
    color: RGB8,
    data: [RGB8; NUM_LEDS],
    delay_max: Milliseconds<u32>,
    position: usize,
}

impl ForwardWave {
    fn new(color: RGB8, brightness: Brightness, speed: Speed) -> Self {
        let data: [RGB8; NUM_LEDS] = [RGB8::default(); NUM_LEDS];
        ForwardWave {
            brightness_max: brightness,
            color,
            data,
            delay_max: Milliseconds::<u32>(speed.value),
            position: 0,
        }
    }
}

impl Effect for ForwardWave {
    fn render(&mut self, ws2812: &mut Ws2812<spi::Spi<microbit::pac::SPI0>>, delay: &mut Timer<microbit::pac::TIMER0>) {
        let wave = [self.brightness_max.value / 10.0, self.brightness_max.value / 6.0 , self.brightness_max.value / 4.0, self.brightness_max.value, self.brightness_max.value / 10.0];
        self.data = [RGB8::default(); NUM_LEDS];
        for i in 0..wave.len() {
            self.data[self.position + i] = RGB8::new((self.color.r as f32 * wave[i]) as u8, (self.color.g as f32 * wave[i]) as u8, (self.color.b as f32 * wave[i]) as u8);
        }
        self.position += 1;
        if self.position >= NUM_LEDS - wave.len() {
            self.position = 0;
        }
        ws2812.write(self.data.iter().cloned()).unwrap();
        delay.delay_ms(self.delay_max.integer() as u16);
    }
}

struct UniColorSparkle {
    brightness_max: Brightness,
    color: RGB8,
    data: [RGB8; NUM_LEDS],
    delay_max: Milliseconds<u32>,
    prng: SmallRng,
}

impl UniColorSparkle {
    fn new(color: RGB8, brightness_max: Brightness, speed: Speed, random_seed: u64) -> Self {
        let data: [RGB8; NUM_LEDS] = [RGB8::default(); NUM_LEDS];
        UniColorSparkle {
            brightness_max,
            color,
            data,
            delay_max: Milliseconds::<u32>(speed.value),
            prng : SmallRng::seed_from_u64(random_seed),
        }
    }
}

impl Effect for UniColorSparkle {
    fn render(&mut self, ws2812: &mut Ws2812<spi::Spi<microbit::pac::SPI0>>, delay: &mut Timer<microbit::pac::TIMER0>) {
        self.data = [RGB8::default(); NUM_LEDS];
        let sparkle_amount = self.prng.gen_range(0..(NUM_LEDS / 10));
        for _ in 0..sparkle_amount {
            let index = self.prng.gen_range(0..NUM_LEDS);
            let brightness = self.prng.gen_range(Brightness::ONE.value..self.brightness_max.value);
            self.data[index] = RGB8::new((self.color.r as f32 * brightness) as u8, (self.color.g as f32 * brightness) as u8, (self.color.b as f32 * brightness) as u8);
        }
        ws2812.write(self.data.iter().cloned()).unwrap();
        delay.delay_ms(self.delay_max.integer() as u16);
    }
}