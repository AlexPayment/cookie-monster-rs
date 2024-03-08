#![no_std]
#![no_main]

use cortex_m_rt::entry;
use microbit::hal::{spi, Timer};
use microbit::hal::gpio::Level;
use microbit::hal::gpio::p0::Parts;
use microbit::Peripherals;
use panic_rtt_target as _;
use rtt_target::{debug_rprintln, rtt_init_print};
use ws2812_spi::Ws2812;

#[entry]
fn main() -> ! {
    rtt_init_print!();

    debug_rprintln!("Hello World!");

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

    let timer = Timer::new(peripherals.TIMER0);

    loop {}
}
