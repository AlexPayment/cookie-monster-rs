#![no_std]
#![no_main]

use animations::{
    Animation, Carrousel, DoubleCarrousel, ForwardWave, MultiColorFadeIn, MultiColorHeartbeat,
    MultiColorSolid, MultiColorSolidRandom, MultiColorSparkle, MultiColorStrand, Settings,
    UniColorFadeIn, UniColorHeartbeat, UniColorSolid, UniColorSparkle, NUM_COLORS, NUM_LEDS,
};
use core::cell::RefCell;
use core::cmp;
use cortex_m_rt::entry;
use defmt::{debug, info};
use defmt_rtt as _;
use microbit::adc::{Adc, AdcConfig};
use microbit::hal::gpio::p0::Parts;
use microbit::hal::gpio::Level;
use microbit::hal::{spi, spim, Timer};
use microbit::{hal, Peripherals};
use nrf52833_hal::saadc::Channel;
use nrf52833_hal::Saadc;
use panic_probe as _;
use smart_leds::RGB8;
use ws2812_spi::Ws2812;

mod animations;

const NUM_ANIMATIONS: usize = 14;

#[entry]
fn main() -> ! {
    // Setup all peripherals and the WS2812 device
    let peripherals = Peripherals::take().unwrap();

    let port0 = Parts::new(peripherals.P0);
    let sck = port0.p0_17.into_push_pull_output(Level::Low).degrade();
    // The SPI MOSI pin is pin 15 on the micro:bit.
    let mosi = port0.p0_13.into_push_pull_output(Level::Low).degrade();
    let miso = port0.p0_01.into_floating_input().degrade();
    let pins = spi::Pins {
        sck: Some(sck),
        miso: Some(miso),
        mosi: Some(mosi),
    };
    let spi = spi::Spi::new(peripherals.SPI0, pins, spi::Frequency::M4, spim::MODE_0);
    let mut ws2812 = Ws2812::new(spi);
    let mut timer = Timer::new(peripherals.TIMER0);

    let mut adc = Adc::new(peripherals.SAADC, AdcConfig::default());
    // This analog pin is the big 0 connector or the pin 0 on the micro:bit.
    let mut animation_pin = port0.p0_02.into_floating_input();
    // This analog pin is the big 1 connector or the pin 1 on the micro:bit.
    let mut brightness_pin = port0.p0_03.into_floating_input();
    // This analog pin is the big 2 connector or the pin 2 on the micro:bit.
    let mut color_pin = port0.p0_04.into_floating_input();
    // This analog pin is the pin 3 on the micro:bit.
    let mut delay_pin = port0.p0_31.into_floating_input();

    // Setup Pseudo Random Number Generator
    let mut rng = hal::Rng::new(peripherals.RNG);

    // Get the maximum value of the potentiometer. Must match the default resolution of the ADC which is 14 bits.
    let max_value = 2u16.pow(14) - 1;
    let default_value = max_value / 2;
    debug!("Max potentiometer value: {}", max_value);

    let data = RefCell::new([RGB8::default(); NUM_LEDS]);

    info!("Initialize animations...");
    let mut carrousel = Carrousel::new(&data, rng.random_u64());
    let mut double_carrousel = DoubleCarrousel::new(&data, rng.random_u64());
    let mut forward_wave = ForwardWave::new(&data);
    let mut multi_color_fade_in = MultiColorFadeIn::new(&data, rng.random_u64());
    let mut multi_color_heartbeat = MultiColorHeartbeat::new(&data, rng.random_u64());
    let mut multi_color_solid = MultiColorSolid::new(&data);
    let mut multi_color_solid_random = MultiColorSolidRandom::new(&data, rng.random_u64());
    let mut multi_color_sparkle = MultiColorSparkle::new(&data, rng.random_u64());
    let mut multi_color_strand = MultiColorStrand::new(&data, rng.random_u64());
    let mut uni_color_fade_in = UniColorFadeIn::new(&data);
    let mut uni_color_front_to_back_wave = animations::UniColorFrontToBackWave::new(&data);
    let mut uni_color_heartbeat = UniColorHeartbeat::new(&data);
    let mut uni_color_solid = UniColorSolid::new(&data);
    let mut uni_color_sparkle = UniColorSparkle::new(&data, rng.random_u64());

    let animations: [&mut dyn Animation; NUM_ANIMATIONS] = [
        &mut multi_color_strand,
        &mut carrousel,
        &mut double_carrousel,
        &mut uni_color_sparkle,
        &mut multi_color_sparkle,
        &mut forward_wave,
        &mut uni_color_fade_in,
        &mut multi_color_fade_in,
        &mut uni_color_front_to_back_wave,
        &mut multi_color_solid,
        &mut multi_color_solid_random,
        &mut uni_color_solid,
        &mut uni_color_heartbeat,
        &mut multi_color_heartbeat,
    ];

    let mut animation_value = default_value as i16;
    let mut previous_animation_index = None;
    let mut brightness_value = default_value as i16;
    let mut color_value = default_value as i16;
    let mut color_index = 9;
    let mut delay_value = default_value as i16;

    let mut settings = Settings::new(
        color_index,
        // Value between 0 and 1
        brightness_value as f32 / max_value as f32,
        calculate_delay(delay_value, max_value),
    );

    info!("Starting main loop...");
    loop {
        animation_value = read_potentiometer(
            &mut adc,
            &mut animation_pin,
            animation_value,
            0,
            max_value as i16,
        );
        brightness_value = read_potentiometer(
            &mut adc,
            &mut brightness_pin,
            brightness_value,
            1,
            max_value as i16,
        );
        color_value =
            read_potentiometer(&mut adc, &mut color_pin, color_value, 0, max_value as i16);
        delay_value =
            read_potentiometer(&mut adc, &mut delay_pin, delay_value, 0, max_value as i16);

        debug!(
            "Animation: {}, Brightness: {}, Color: {}, Delay: {}",
            animation_value, brightness_value, color_value, delay_value
        );

        let animation_index = calculate_index(animation_value, max_value, NUM_ANIMATIONS);
        color_index = calculate_index(color_value, max_value, NUM_COLORS);

        if previous_animation_index.is_none()
            || animation_index != previous_animation_index.unwrap()
        {
            animations[animation_index].reset();
            previous_animation_index = Some(animation_index);
        }

        // Value between 0 and 1
        settings.set_brightness(calculate_brightness(brightness_value, max_value));
        settings.set_color_index(color_index);
        settings.set_delay(calculate_delay(delay_value, max_value));

        debug!("{:?}", settings);
        debug!("Current animation: {}", animation_index);

        animations[animation_index].render(&mut ws2812, &mut timer, &settings);
    }
}

/// Calculate the brightness based on the value of the potentiometer.
///
/// The value is between 0 and 1.
fn calculate_brightness(value: i16, max_value: u16) -> f32 {
    value as f32 / max_value as f32
}

/// Calculate the delay in milliseconds based on the value of the potentiometer.
fn calculate_delay(value: i16, max_value: u16) -> u32 {
    cmp::max((value as f32 / max_value as f32 * 1000.0) as u32, 1)
}

fn calculate_index(value: i16, max_value: u16, num_values: usize) -> usize {
    let index = (value as f32 / max_value as f32 * num_values as f32) as usize;
    cmp::min(index, num_values - 1)
}

fn read_potentiometer<PIN: Channel>(
    adc: &mut Saadc, pin: &mut PIN, default_value: i16, min_value: i16, max_value: i16,
) -> i16 {
    cmp::max(
        min_value,
        cmp::min(adc.read_channel(pin).unwrap_or(default_value), max_value),
    )
}
