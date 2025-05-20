#![no_std]
#![no_main]

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::gpio::Pin;
use esp_hal::timer::timg::TimerGroup;
use {esp_backtrace as _, esp_println as _};

use crate::controls::animation_button_task;

static ANIMATION: usize = 0;
const NUM_ANIMATIONS: usize = 14;

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // TODO: Check if the CPU clock could be set lower to save power
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let timer0 = TimerGroup::new(peripherals.TIMG1);
    esp_hal_embassy::init(timer0.timer0);

    info!("Embassy initialized!");

    // GPIO15 is the Q1 pin on the board, it's pulled low. Which means the button should also be
    // connected to a 3.3 V pin.
    let animation_button = peripherals.GPIO15.degrade();
    // Spawn the animation button task
    unwrap!(spawner.spawn(animation_button_task(
        animation_button,
        ANIMATION,
        NUM_ANIMATIONS
    )));

    loop {
        info!("Hello world!");
        Timer::after(Duration::from_secs(1)).await;
    }
}

mod controls;
