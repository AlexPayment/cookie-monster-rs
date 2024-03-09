#![no_std]
#![no_main]

use cortex_m_rt::entry;
use microbit::{Board, hal};
use microbit::hal::gpio::Level;
use microbit::hal::spi;
use panic_rtt_target as _;
use rand::{Rng, SeedableRng};
use rand::rngs::SmallRng;
use rtt_target::{debug_rprintln, rtt_init_print};

use cookie_monster::CookieMonster;

use crate::controls::init_buttons;

mod controls;
mod cookie_monster;

#[entry]
fn main() -> ! {
    rtt_init_print!();

    // Setup all peripherals and the WS2812 device
    let board = Board::take().unwrap();
    let sck = board.pins.p0_17.into_push_pull_output(Level::Low).degrade();
    // The SPI MOSI pin is pin 15 on the micro:bit.
    let mosi = board.pins.p0_13.into_push_pull_output(Level::Low).degrade();
    let miso = board.pins.p0_01.into_floating_input().degrade();
    let pins = spi::Pins {
        sck,
        miso: Some(miso),
        mosi: Some(mosi),
    };
    // let spi = spi::Spi::new(peripherals.SPI0, pins, spi::Frequency::M4, spi::MODE_0);
    // let ws2812 = Ws2812::new(spi);

    // Setup Pseudo Random Number Generator
    let mut rng = hal::Rng::new(board.RNG);
    let mut prng = SmallRng::seed_from_u64(rng.random_u64());

    let mut randos = [0x00u8; 64];
    prng.fill(&mut randos);
    debug_rprintln!("Randoms: {:?}", randos);

    let cookie_monster = CookieMonster::new(board.display_pins, board.TIMER0);
    init_buttons(board.GPIOTE, board.buttons, cookie_monster);

    loop {}
}
