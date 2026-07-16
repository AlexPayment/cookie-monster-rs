#![no_std]
#![no_main]

use crate::input::{
    ANALOG_DEFAULT_VALUE, ANALOG_MAXIMUM_VALUE, analog_sensors_task, animation_button_task,
    color_button_task,
};
use crate::led::SpiConfig;
use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_nrf::Peri;
use embassy_nrf::config::Config;
use embassy_nrf::gpio::AnyPin;
use embassy_nrf::peripherals::{RNG, SAADC, SPI2, SPI3};
use embassy_nrf::saadc::{AnyInput, Input};
use embassy_time::Delay;
use embedded_hal_async::delay::DelayNs;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let peripherals = embassy_nrf::init(Config::default());

    info!("Embassy initialized!");

    let pins = Pins {
        // GPIO port 0 pin 14 corresponds to pin 5 on the board, it's pull up. Which means a button
        // should be connected to a ground pin. This pin is normally used for button A on the
        // board.
        animation: peripherals.P0_14.into(),

        // GPIO port 0 pin 2 is an analog input corresponding to the big "0" connector or pin 0 on
        // the board.
        brightness: peripherals.P0_02.degrade_saadc(),

        // GPIO port 0 pin 23 corresponds to pin 11 on the board, it's pull up. Which means a button
        // should be connected to a ground pin. This pin is normally used for button B on the
        // board.
        color: peripherals.P0_23.into(),

        // GPIO port 0 pin 3 is an analog input corresponding to the big "1" connector or pin 1 on
        // the board.
        delay: peripherals.P0_03.degrade_saadc(),

        // GPIO port 0 pin 13 corresponds to pin 15 on the board.
        led_1: peripherals.P0_13.into(),

        // GPIO port 0 pin 10 corresponds to pin 8 on the board.
        led_2: peripherals.P0_10.into(),

        // GPIO port 0 pin 17 corresponds to pin 13 on the board.
        sck_1: peripherals.P0_17.into(),

        // GPIO port 1 pin 2 corresponds to pin 16 on the board.
        sck_2: peripherals.P1_02.into(),
    };

    spawn_all_tasks(
        &spawner,
        peripherals.SAADC,
        peripherals.RNG,
        peripherals.SPI2,
        peripherals.SPI3,
        pins,
    );

    let mut delay = Delay;

    info!("Starting main loop...");
    loop {
        delay.delay_ms(1_000).await;
    }
}

/// Represents the pins used for both input and output.
struct Pins<'a> {
    animation: Peri<'a, AnyPin>,
    brightness: AnyInput<'a>,
    color: Peri<'a, AnyPin>,
    delay: AnyInput<'a>,
    led_1: Peri<'a, AnyPin>,
    led_2: Peri<'a, AnyPin>,
    sck_1: Peri<'a, AnyPin>,
    sck_2: Peri<'a, AnyPin>,
}

/// Spawns all the tasks for the inputs and LEDs.
fn spawn_all_tasks(
    spawner: &Spawner, adc: Peri<'static, SAADC>, rng: Peri<'static, RNG>,
    spi_1: Peri<'static, SPI2>, spi_2: Peri<'static, SPI3>, pins: Pins<'static>,
) {
    info!("Spawning all tasks...");

    // Spawn the analog sensors task
    spawner.spawn(unwrap!(analog_sensors_task(
        adc,
        pins.brightness,
        pins.delay
    )));

    // Spawn the animation button task
    spawner.spawn(unwrap!(animation_button_task(pins.animation)));

    // Spawn the color button task
    spawner.spawn(unwrap!(color_button_task(pins.color)));

    // Spawn the LED task
    spawner.spawn(unwrap!(led::led_task(
        rng,
        SpiConfig {
            spim: spi_1,
            sck: pins.sck_1,
            led_pin: pins.led_1,
        },
        SpiConfig {
            spim: spi_2,
            sck: pins.sck_2,
            led_pin: pins.led_2,
        },
        ANALOG_DEFAULT_VALUE,
        ANALOG_MAXIMUM_VALUE
    )));
}

mod input;
mod led;
