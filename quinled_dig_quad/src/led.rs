use crate::EmbassyTimer;
use crate::input::SettingsMutex;
use cookie_monster_common::animations::carrousel::Carrousel;
use cookie_monster_common::animations::{Animation, create_data};
use defmt::info;
use esp_hal::gpio::AnyPin;
use esp_hal::peripherals::{RNG, SPI2};
use esp_hal::rng::Rng;
use esp_hal::spi::master::{Config as SpiConfig, Spi};
use esp_hal::time::Rate;
use ws2812_spi::Ws2812;

pub const NUM_ANIMATIONS: usize = 1;

#[embassy_executor::task]
pub async fn led_task(
    rng: RNG, spi: SPI2, led: AnyPin, animation: usize, settings_mutex: &'static SettingsMutex,
) {
    // According to the ws2812_spi documentation, the SPI frequency must be between 2 and 3.8 MHz.
    let spi = Spi::new(spi, SpiConfig::default().with_frequency(Rate::from_mhz(2)))
        .unwrap()
        .with_mosi(led)
        .into_async();

    let mut ws2812 = Ws2812::new(spi);
    let mut timer = EmbassyTimer();

    // Setup Pseudo Random Number Generator
    let mut rng = Rng::new(rng);

    let data = create_data();

    // TODO: initialize all the animations
    info!("Initialize animations...");
    let carrousel = Carrousel::new(&data, rng.random() as u64);

    let mut animations: [Animation; NUM_ANIMATIONS] = [Animation::Carrousel(carrousel)];

    loop {
        if let Some(animation) = animations.get_mut(animation) {
            let settings = settings_mutex.lock().await;
            animation.update(&settings);
            animation.render(&mut ws2812, &mut timer, &settings);
        }
    }
}
