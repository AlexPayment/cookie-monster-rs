#![no_std]
#![no_main]

use crate::input::{
    BrightnessPin, DelayPin, analog_sensors_task, animation_button_task, color_button_task,
};
use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_time::Delay;
use embedded_hal_async::delay::DelayNs;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{AnyPin, Pin};
use esp_hal::peripherals::{ADC2, RNG};
use esp_hal::spi::AnySpi;
use esp_hal::timer::timg::TimerGroup;
use {esp_backtrace as _, esp_println as _};

// The ADC resolution is 12 bits, which means the maximum value is 4095 (2^12 - 1).
const ADC_MAX_VALUE: u16 = 2u16.pow(ADC_RESOLUTION) - 1;
const ADC_RESOLUTION: u32 = 12;
// The default analog value is set to half of the maximum value, which is 2048.
const DEFAULT_ANALOG_VALUE: u16 = ADC_MAX_VALUE / 2;

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // TODO: Check if the CPU clock could lowered to save power
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let timer0 = TimerGroup::new(peripherals.TIMG1);
    esp_hal_embassy::init(timer0.timer0);

    info!("Embassy initialized!");

    let pins = Pins {
        // GPIO02 is the Q3 pin on the board, it's pull high. Which means a button should be
        // connected to a ground pin. A potentiometer shouldn't be connected to anything higher than
        // 3.3 V. This pin is on ADC2 channel 2.
        animation: peripherals.GPIO2.degrade(),

        // GPIO15 is the Q1 pin on the board, it's pull low. Which means a button should be
        // connected to a 3.3 or 5 V pin. A potentiometer shouldn't be connected to anything higher
        // than 3.3 V. This pin is on ADC2 channel 3.
        brightness: peripherals.GPIO15,

        // GPIO32 is the Q4 pin on the board, it's pull high. Which means a button also be
        // connected to a ground pin. A potentiometer shouldn't be connected to anything higher than
        // 3.3 V. This pin is on ADC1 channel 4.
        color: peripherals.GPIO32.degrade(),

        // GPIO12 is the Q2 pin on the board, it's pull low. Which means a button also be
        // connected to a 3.3 or 5 V pin. A potentiometer shouldn't be connected to anything higher
        // than 3.3 V. This pin is on ADC2 channel 5.
        delay: peripherals.GPIO12,

        // Pin that's labeled LED1 on the board.
        led: peripherals.GPIO16.degrade(),
    };

    spawn_all_tasks(
        &spawner,
        peripherals.ADC2,
        peripherals.RNG,
        // It's unclear why SPI2 is used instead of another SPI peripheral, but this is the one seen
        // in many examples.
        peripherals.SPI2.into(),
        pins,
    );

    let mut delay = Delay;

    loop {
        delay.delay_ms(1_000).await
    }
}

/// Represents the pins used for both input and output.
struct Pins<'a> {
    animation: AnyPin<'a>,
    brightness: BrightnessPin<'a>,
    color: AnyPin<'a>,
    delay: DelayPin<'a>,
    led: AnyPin<'a>,
}

/// Spawns all the tasks for the inputs and LEDs.
fn spawn_all_tasks(
    spawner: &Spawner, adc: ADC2<'static>, rng: RNG<'static>, spi: AnySpi<'static>,
    pins: Pins<'static>,
) {
    info!("Spawning all tasks...");

    // Spawn the animation button task
    unwrap!(spawner.spawn(animation_button_task(pins.animation)));

    // Spawn the color button task
    unwrap!(spawner.spawn(color_button_task(pins.color)));

    // Spawn the analog sensors task
    unwrap!(spawner.spawn(analog_sensors_task(adc, pins.brightness, pins.delay)));

    // Spawn the LED task
    unwrap!(spawner.spawn(led::led_task(
        rng,
        spi,
        pins.led,
        DEFAULT_ANALOG_VALUE,
        ADC_MAX_VALUE
    )));
}

mod input;
mod led;
