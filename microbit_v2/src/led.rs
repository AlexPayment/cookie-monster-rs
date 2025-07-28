use cookie_monster_common::animations::{
    DEFAULT_COLOR_INDEX, NUM_ANIMATIONS, NUM_COLORS, NUM_LEDS, Settings, create_data,
    initialize_animations,
};
use cookie_monster_common::signal::{
    ANIMATION_CHANGED_SIGNAL, BRIGHTNESS_READ_SIGNAL, COLOR_CHANGED_SIGNAL, DELAY_READ_SIGNAL,
};
use defmt::{debug, info};
use embassy_nrf::gpio::AnyPin;
use embassy_nrf::peripherals::{RNG, SPI2};
use embassy_nrf::rng::Rng;
use embassy_nrf::spim::{Config, Frequency, Spim};
use embassy_nrf::{Peri, bind_interrupts, rng, spim};
use embassy_time::Delay;
use rand::SeedableRng;
use rand::rngs::SmallRng;
use ws2812_spi::prerendered::Ws2812;

bind_interrupts!(struct Irqs {
    RNG => rng::InterruptHandler<RNG>;
    SPI2 => spim::InterruptHandler<SPI2>;
});

#[embassy_executor::task]
pub async fn led_task(
    rng: Peri<'static, RNG>, spi: Peri<'static, SPI2>, sck: Peri<'static, AnyPin>,
    led: Peri<'static, AnyPin>, default_analog_value: u16, max_analog_value: u16,
) {
    info!("Starting LED task...");

    let mut config = Config::default();
    config.frequency = Frequency::M4;
    let spi = Spim::new_txonly(spi, Irqs, sck, led, config);

    let mut buffer = [0; NUM_LEDS * 12];
    let mut ws2812 = Ws2812::new(spi, &mut buffer);

    // Setup Pseudo Random Number Generator
    let mut prng = setup_prng(rng).await;

    let data = create_data();

    let mut animation_index = 0;
    let mut animations = initialize_animations(&data, &mut prng);

    info!("Creating default animation settings");
    let mut settings = Settings::new(
        DEFAULT_COLOR_INDEX,
        default_analog_value,
        default_analog_value,
        max_analog_value,
        NUM_COLORS,
    );

    let mut delay = Delay;

    loop {
        if let Some(()) = ANIMATION_CHANGED_SIGNAL.try_take() {
            info!("Animation changed signal received");
            animation_index = (animation_index + 1) % NUM_ANIMATIONS;
            animations[animation_index].reset();
        }

        if let Some(brightness) = BRIGHTNESS_READ_SIGNAL.try_take() {
            settings.set_brightness(brightness);
        }

        if let Some(()) = COLOR_CHANGED_SIGNAL.try_take() {
            info!("Color changed signal received");
            settings.set_color_index((settings.color_index() + 1) % NUM_COLORS);
        }

        if let Some(delay) = DELAY_READ_SIGNAL.try_take() {
            settings.set_delay(delay);
        }

        debug!("Updating animation data");
        animations[animation_index].update(&settings);

        debug!("Rendering animation");
        animations[animation_index]
            .render(&mut ws2812, &mut delay, &settings)
            .await;
    }
}

async fn setup_prng(rng: Peri<'static, RNG>) -> SmallRng {
    let mut rng = Rng::new(rng, Irqs);
    let mut seed = [0; 8];
    rng.fill_bytes(&mut seed).await;
    let seed = u64::from_le_bytes(seed);
    SmallRng::seed_from_u64(seed)
}
