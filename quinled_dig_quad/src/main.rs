#![no_std]
#![no_main]

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::analog::adc::{Adc, AdcConfig, Attenuation};
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{AnyPin, Pin};
use esp_hal::timer::timg::TimerGroup;
use nb::block;
use {esp_backtrace as _, esp_println as _};

use crate::controls::{animation_button_task, color_button_task};

static ANIMATION: usize = 0;
static COLOR: usize = 0;
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

    // The ESP32 ADC has a resolution of 12 bits, which means the maximum value is 4095.
    let mut adc2_config = AdcConfig::default();
    // Because the brightness potentiometer is connected to the 3.3 V pin, we need to set the
    // attenuation to 11 dB to cover the 0 to 3.3 V range.
    let mut brightness_pin = adc2_config.enable_pin(brightness_pin, Attenuation::_11dB);
    // Because the delay potentiometer is connected to the 3.3 V pin, we need to set the
    // attenuation to 11 dB to cover the 0 to 3.3 V range.
    let mut delay_pin = adc2_config.enable_pin(delay_pin, Attenuation::_11dB);
    let mut adc2 = Adc::new(peripherals.ADC2, adc2_config);

    spawn_control_tasks(&spawner, animation_pin, color_pin);

    loop {
        let brightness_value = block!(adc2.read_oneshot(&mut brightness_pin)).unwrap();
        let delay_value = block!(adc2.read_oneshot(&mut delay_pin)).unwrap();
        info!("Brightness: {}, Delay: {}", brightness_value, delay_value);
        Timer::after(Duration::from_millis(500)).await;
    }
}

/// Spawns the tasks for all the manual controls.
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
