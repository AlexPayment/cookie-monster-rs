#![no_std]
#![no_main]

use cortex_m_rt::entry;
use microbit::{hal, Peripherals};
use microbit::hal::{spi, Timer};
use microbit::hal::gpio::Level;
use microbit::hal::gpio::p0::Parts;
use panic_rtt_target as _;
use rand::{Rng, SeedableRng};
use rand::rngs::SmallRng;
use rtt_target::{debug_rprintln, rtt_init_print};
use ws2812_spi::Ws2812;

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
    let ws2812 = Ws2812::new(spi);

    // Setup Pseudo Random Number Generator
    let mut rng = hal::Rng::new(peripherals.RNG);
    let mut prng = SmallRng::seed_from_u64(rng.random_u64());

    let mut randos = [0x00u8; 64];
    prng.fill(&mut randos);
    debug_rprintln!("Randoms: {:?}", randos);

    let timer = Timer::new(peripherals.TIMER0);

    loop {}
}
