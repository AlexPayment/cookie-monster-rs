#![no_std]
#![no_main]

use core::cell::RefCell;
use cortex_m_rt::entry;
use effects::{Effect, ForwardWave, Settings, Speed, UniColorSparkle, NUM_LEDS};
use microbit::hal::gpio::p0::Parts;
use microbit::hal::gpio::Level;
use microbit::hal::{spi, Timer};
use microbit::{hal, Peripherals};
use panic_rtt_target as _;
use rtt_target::rtt_init_print;
use smart_leds::RGB8;
use ws2812_spi::Ws2812;

mod controls;
mod cookie_monster;
mod effects;

#[entry]
fn main() -> ! {
    rtt_init_print!();

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
    let settings = Settings::new(
        RGB8::new(0x00, 0x00, 0xff),
        Speed::SLOW,
    );

    let mut uni_color_sparkle = UniColorSparkle::new(&data, &settings, rng.random_u64());
    let mut forward_wave = ForwardWave::new(&data, &settings);

    let effect: [&mut dyn Effect; 2] = [&mut uni_color_sparkle, &mut forward_wave];

    loop {
        effect[0].render(&mut ws2812, &mut delay);
    }
}
