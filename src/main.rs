#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_rtt_target as _;
use rtt_target::{debug_rprintln, rtt_init_print};

#[entry]
fn main() -> ! {
    rtt_init_print!();

    debug_rprintln!("Hello World!");

    loop {}
}
