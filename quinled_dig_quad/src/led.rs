use crate::signal::{
    ANIMATION_CHANGED_SIGNAL, BRIGHTNESS_READ_SIGNAL, COLOR_CHANGED_SIGNAL, DELAY_READ_SIGNAL,
};
use cookie_monster_common::animations::{
    DEFAULT_COLOR_INDEX, NUM_ANIMATIONS, NUM_COLORS, Settings, create_data, initialize_animations,
};
use embassy_time::Delay;
use esp_hal::gpio::AnyPin;
use esp_hal::peripherals::{RNG, SPI2};
use esp_hal::rng::Rng;
use esp_hal::spi::master::{Config as SpiConfig, Spi};
use esp_hal::time::Rate;
use rand::SeedableRng;
use rand::rngs::SmallRng;
use ws2812_spi::Ws2812;

#[embassy_executor::task]
pub async fn led_task(
    rng: RNG, spi: SPI2, led: AnyPin, default_analog_value: u16, max_analog_value: u16,
) {
    // According to the ws2812_spi documentation, the SPI frequency must be between 2 and 3.8 MHz.
    let spi = Spi::new(spi, SpiConfig::default().with_frequency(Rate::from_mhz(2)))
        .unwrap()
        .with_mosi(led)
        .into_async();

    let mut ws2812 = Ws2812::new(spi);
    let mut delay = Delay;

    // Setup Pseudo Random Number Generator
    let mut rng = Rng::new(rng);
    let mut prng = SmallRng::seed_from_u64(rng.random() as u64);

    let data = create_data();

    let mut animation_index = 0;
    let mut animations = initialize_animations(&data, &mut prng);

    let mut settings = Settings::new(
        DEFAULT_COLOR_INDEX,
        default_analog_value,
        default_analog_value,
        max_analog_value,
        NUM_COLORS,
    );

    loop {
        if let Some(()) = ANIMATION_CHANGED_SIGNAL.try_take() {
            animation_index = (animation_index + 1) % NUM_ANIMATIONS;
            animations[animation_index].reset();
        }

        if let Some(brightness) = BRIGHTNESS_READ_SIGNAL.try_take() {
            settings.set_brightness(brightness);
        }

        if let Some(()) = COLOR_CHANGED_SIGNAL.try_take() {
            settings.set_color_index((settings.color_index() + 1) % NUM_COLORS);
        }

        if let Some(delay) = DELAY_READ_SIGNAL.try_take() {
            settings.set_delay(delay);
        }

        animations[animation_index].update(&settings);
        animations[animation_index].render(&mut ws2812, &mut delay, &settings);
    }
}
