#![no_std]
#![no_main]

use crate::input::{
    BrightnessPin, DelayPin, analog_sensors_task, animation_button_task, color_button_task,
};
use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{AnyPin, Pin};
use esp_hal::peripherals::ADC2;
use esp_hal::timer::timg::TimerGroup;
use {esp_backtrace as _, esp_println as _};

type AnimationChangedSignal = Signal<CriticalSectionRawMutex, ()>;
type BrightnessReadSignal = Signal<CriticalSectionRawMutex, u16>;
type ColorChangedSignal = Signal<CriticalSectionRawMutex, ()>;
type DelayReadSignal = Signal<CriticalSectionRawMutex, u16>;

static ANIMATION_CHANGED_SIGNAL: AnimationChangedSignal = AnimationChangedSignal::new();
static BRIGHTNESS_READ_SIGNAL: BrightnessReadSignal = BrightnessReadSignal::new();
static COLOR_CHANGED_SIGNAL: ColorChangedSignal = ColorChangedSignal::new();
static DELAY_READ_SIGNAL: DelayReadSignal = DelayReadSignal::new();

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // TODO: Check if the CPU clock could lowered to save power
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let timer0 = TimerGroup::new(peripherals.TIMG1);
    esp_hal_embassy::init(timer0.timer0);

    info!("Embassy initialized!");

    // GPIO02 is the Q3 pin on the board, it's pulled high. Which means a button should be
    // connected to a ground pin. A potentiometer shouldn't be connected to anything higher than
    // 3.3 V. This pin is on ADC2 channel 2.
    let animation_pin = peripherals.GPIO2.degrade();

    // GPIO15 is the Q1 pin on the board, it's pulled low. Which means a button should be
    // connected to a 3.3 or 5 V pin. A potentiometer shouldn't be connected to anything higher than
    // 3.3 V. This pin is on ADC2 channel 3.
    let brightness_pin = peripherals.GPIO15;

    // GPIO32 is the Q4 pin on the board, it's pulled high. Which means a button also be
    // connected to a ground pin. A potentiometer shouldn't be connected to anything higher than
    // 3.3 V. This pin is on ADC1 channel 4.
    let color_pin = peripherals.GPIO32.degrade();

    // GPIO12 is the Q2 pin on the board, it's pulled low. Which means a button also be
    // connected to a 3.3 or 5 V pin. A potentiometer shouldn't be connected to anything higher than
    // 3.3 V. This pin is on ADC2 channel 5.
    let delay_pin = peripherals.GPIO12;

    spawn_control_tasks(
        &spawner,
        peripherals.ADC2,
        animation_pin,
        brightness_pin,
        color_pin,
        delay_pin,
    );

    loop {
        Timer::after(Duration::from_millis(500)).await;
    }
}

/// Spawns the tasks for all the manual controls.
fn spawn_control_tasks(
    spawner: &Spawner, adc: ADC2, animation_pin: AnyPin, brightness_pin: BrightnessPin,
    color_pin: AnyPin, delay_pin: DelayPin,
) {
    // Spawn the animation button task
    unwrap!(spawner.spawn(animation_button_task(
        animation_pin,
        &ANIMATION_CHANGED_SIGNAL
    )));

    // Spawn the color button task
    unwrap!(spawner.spawn(color_button_task(color_pin, &COLOR_CHANGED_SIGNAL)));

    // Spawn the analog sensors task
    unwrap!(spawner.spawn(analog_sensors_task(
        adc,
        brightness_pin,
        delay_pin,
        &BRIGHTNESS_READ_SIGNAL,
        &DELAY_READ_SIGNAL
    )));
}

mod input;
