#![no_std]
#![no_main]

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{AnyPin, Pin};
use esp_hal::timer::timg::TimerGroup;
use {esp_backtrace as _, esp_println as _};

use crate::controls::{animation_button_task, color_button_task};

static ANIMATION: usize = 0;
static COLOR: usize = 0;
const NUM_ANIMATIONS: usize = 14;
const NUM_COLORS: usize = 11;

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // TODO: Check if the CPU clock could be set lower to save power
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let timer0 = TimerGroup::new(peripherals.TIMG1);
    esp_hal_embassy::init(timer0.timer0);

    info!("Embassy initialized!");

    // GPIO15 is the Q1 pin on the board, it's pulled low. Which means the button should also be
    // connected to a 3.3 or 5 V pin.
    let animation_pin = peripherals.GPIO15.degrade();
    // GPIO12 is the Q2 pin on the board, it's pulled low. Which means the button should also be
    // connected to a 3.3 or 5 V pin.
    let color_pin = peripherals.GPIO12.degrade();

    spawn_control_tasks(&spawner, animation_pin, color_pin);

    loop {
        info!("Hello world!");
        Timer::after(Duration::from_secs(1)).await;
    }
}

/// Spawns the tasks for the animation, brightness, color and delay controls.
fn spawn_control_tasks(spawner: &Spawner, animation_pin: AnyPin, color_pin: AnyPin) {
    // Spawn the animation button task
    unwrap!(spawner.spawn(animation_button_task(
        animation_pin,
        ANIMATION,
        NUM_ANIMATIONS
    )));

    // Spawn the color button task
    unwrap!(spawner.spawn(color_button_task(color_pin, COLOR, NUM_COLORS)));
}

mod controls;
