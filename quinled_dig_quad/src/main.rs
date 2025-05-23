#![no_std]
#![no_main]

use crate::controls::{
    BrightnessPin, DelayPin, SettingsMutex, analog_sensors_task, animation_button_task,
    color_button_task,
};
use cookie_monster_common::animations::{DEFAULT_COLOR_INDEX, Settings};
use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_sync::mutex::Mutex;
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{AnyPin, Pin};
use esp_hal::peripherals::ADC2;
use esp_hal::timer::timg::TimerGroup;
use static_cell::StaticCell;
use {esp_backtrace as _, esp_println as _};

static ANIMATION: usize = 0;
static SETTINGS: StaticCell<SettingsMutex> = StaticCell::new();
const ADC_MAX_VALUE: u16 = 2u16.pow(ADC_RESOLUTION) - 1;
const ADC_RESOLUTION: u32 = 12;
const DEFAULT_ANALOG_VALUE: u16 = ADC_MAX_VALUE / 2;
const NUM_ANIMATIONS: usize = 14;
const NUM_COLORS: usize = 11;

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

    let settings = Settings::new(
        DEFAULT_COLOR_INDEX,
        DEFAULT_ANALOG_VALUE,
        DEFAULT_ANALOG_VALUE,
        ADC_MAX_VALUE,
        NUM_COLORS,
    );
    let settings_mutex = SETTINGS.init(Mutex::new(settings));

    spawn_control_tasks(
        &spawner,
        peripherals.ADC2,
        animation_pin,
        brightness_pin,
        color_pin,
        delay_pin,
        settings_mutex,
    );

    loop {
        Timer::after(Duration::from_millis(500)).await;
    }
}

/// Spawns the tasks for all the manual controls.
fn spawn_control_tasks(
    spawner: &Spawner, adc: ADC2, animation_pin: AnyPin, brightness_pin: BrightnessPin,
    color_pin: AnyPin, delay_pin: DelayPin, settings_mutex: &'static SettingsMutex,
) {
    // Spawn the animation button task
    unwrap!(spawner.spawn(animation_button_task(
        animation_pin,
        ANIMATION,
        NUM_ANIMATIONS
    )));

    // Spawn the color button task
    unwrap!(spawner.spawn(color_button_task(color_pin, settings_mutex)));

    // Spawn the analog sensors task
    unwrap!(spawner.spawn(analog_sensors_task(
        adc,
        brightness_pin,
        delay_pin,
        settings_mutex
    )));
}

mod controls;
