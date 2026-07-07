#![no_std]
#![no_main]

use crate::input::{
    ANALOG_DEFAULT_VALUE, ANALOG_MAXIMUM_VALUE, BrightnessPin, DelayPin, analog_sensors_task,
    animation_button_task, color_button_task,
};
use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_time::Delay;
use embedded_hal_async::delay::DelayNs;
use esp_hal::clock::CpuClock;
use esp_hal::dma::AnySpiDmaChannel;
use esp_hal::gpio::{AnyPin, Pin};
use esp_hal::interrupt::software::SoftwareInterruptControl;
use esp_hal::peripherals::{ADC2, RMT};
use esp_hal::spi::master::AnySpi;
use esp_hal::timer::timg::TimerGroup;
use {esp_backtrace as _, esp_println as _};

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(spawner: Spawner) {
    // TODO: Check if the CPU clock could be lowered to save power
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let timer0 = TimerGroup::new(peripherals.TIMG1);
    let software_interrupt = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timer0.timer0, software_interrupt.software_interrupt0);

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

        // GPIO32 is the Q4 pin on the board, it's pull high. Which means a button should be
        // connected to a ground pin. A potentiometer shouldn't be connected to anything higher than
        // 3.3 V. This pin is on ADC1 channel 4.
        color: peripherals.GPIO32.degrade(),

        // GPIO12 is the Q2 pin on the board, it's pull low. Which means a button should be
        // connected to a 3.3 or 5 V pin. A potentiometer shouldn't be connected to anything higher
        // than 3.3 V. This pin is on ADC2 channel 5.
        delay: peripherals.GPIO12,

        // Pin that's labeled LED1 on the board.
        led1: peripherals.GPIO16.degrade(),

        // Pin that's labeled LED2 on the board.
        led2: peripherals.GPIO3.degrade(),

        // Pin that's labeled LED2 on the board.
        led3: peripherals.GPIO1.degrade(),

        // Pin that's labeled LED2 on the board.
        led4: peripherals.GPIO4.degrade(),
    };

    spawn_all_tasks(
        &spawner,
        peripherals.ADC2,
        // On ESP32 there are four SPIs, but SPI0 and SPI1 are internally reserved for SPI flash
        // memory. That only leaves SPI2 and SPI3 available.
        peripherals.SPI2.into(),
        peripherals.DMA_SPI2.into(),
        peripherals.RMT,
        pins,
    );

    let mut delay = Delay;

    loop {
        delay.delay_ms(1_000).await;
    }
}

/// Represents the pins used for both input and output.
struct Pins<'a> {
    animation: AnyPin<'a>,
    brightness: BrightnessPin<'a>,
    color: AnyPin<'a>,
    delay: DelayPin<'a>,
    led1: AnyPin<'a>,
    led2: AnyPin<'a>,
    led3: AnyPin<'a>,
    led4: AnyPin<'a>,
}

/// Spawns all the tasks for the inputs and LEDs.
fn spawn_all_tasks(
    spawner: &Spawner, adc: ADC2<'static>, spi: AnySpi<'static>,
    dma_channel: AnySpiDmaChannel<'static>, rmt: RMT<'static>, pins: Pins<'static>,
) {
    info!("Spawning all tasks...");

    // Spawn the animation button task
    spawner.spawn(unwrap!(animation_button_task(pins.animation)));

    // Spawn the color button task
    spawner.spawn(unwrap!(color_button_task(pins.color)));

    // Spawn the analog sensors task
    spawner.spawn(unwrap!(analog_sensors_task(
        adc,
        pins.brightness,
        pins.delay
    )));

    // Spawn the LED task
    spawner.spawn(unwrap!(led::led_task(
        spi,
        dma_channel,
        rmt,
        pins.led1,
        ANALOG_DEFAULT_VALUE,
        ANALOG_MAXIMUM_VALUE
    )));
}

mod input;
mod led;
