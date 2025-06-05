use cookie_monster_common::animations::{
    DEFAULT_COLOR_INDEX, NUM_ANIMATIONS, NUM_COLORS, Settings, create_data, initialize_animations,
};
use cookie_monster_common::signal::{
    ANIMATION_CHANGED_SIGNAL, BRIGHTNESS_READ_SIGNAL, COLOR_CHANGED_SIGNAL, DELAY_READ_SIGNAL,
};
use defmt::{debug, info};
use embassy_nrf::gpio::AnyPin;
use embassy_nrf::peripherals::{RNG, SPI2};
use embassy_nrf::rng::Rng;
use embassy_nrf::{bind_interrupts, rng, spis};
use embassy_time::Delay;
use rand::SeedableRng;
use rand::rngs::SmallRng;

bind_interrupts!(struct Irqs {
    RNG => rng::InterruptHandler<RNG>;
    SPI2 => spis::InterruptHandler<SPI2>;
});

#[embassy_executor::task]
pub async fn led_task(
    rng: RNG, spi: SPI2, led: AnyPin, default_analog_value: u16, max_analog_value: u16,
) {
    info!("Starting LED task...");

    // TODO: Setup the SPI

    // Setup Pseudo Random Number Generator
    let mut rng = Rng::new(rng, Irqs);
    let mut seed = [0; 8];
    rng.fill_bytes(&mut seed).await;
    let seed = u64::from_le_bytes(seed);
    let mut prng = SmallRng::seed_from_u64(seed);

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
        // TODO: Render the animation
    }
}
