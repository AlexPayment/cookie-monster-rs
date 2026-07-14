use cookie_monster_common::animations::{
    Animation, AnimationKind, COLORS_INDEX_DEFAULT, COLORS_TOTAL, LEDS_FIRST_SECTION,
    LEDS_SECOND_SECTION, Settings, create_data, reset_data,
};
use cookie_monster_common::signal::{
    ANIMATION_CHANGED_SIGNAL, BRIGHTNESS_READ_SIGNAL, COLOR_CHANGED_SIGNAL, DELAY_READ_SIGNAL,
};
use defmt::{debug, info};
use embassy_nrf::gpio::AnyPin;
use embassy_nrf::peripherals::{RNG, SPI2, SPI3};
use embassy_nrf::rng::Rng;
use embassy_nrf::spim::{Config, Frequency, Instance, Spim};
use embassy_nrf::{Peri, bind_interrupts, rng, spim};
use embassy_time::Delay;
use rand::SeedableRng;
use rand::rngs::SmallRng;
use ws2812_spi::prerendered::Ws2812;

bind_interrupts!(struct Irqs {
    RNG => rng::InterruptHandler<RNG>;
    SPI2 => spim::InterruptHandler<SPI2>;
    SPIM3 => spim::InterruptHandler<SPI3>;
});

// 12 is calculated by knowing that we're using 8 bits per color (3 bytes total), and that
// ws2812_spi converts each color byte to 4 SPI bytes.
const FIRST_SECTION_BUFFERS_SIZE: usize = LEDS_FIRST_SECTION * 12;

// 12 is calculated by knowing that we're using 8 bits per color (3 bytes total), and that
// ws2812_spi converts each color byte to 4 SPI bytes.
const SECOND_SECTION_BUFFERS_SIZE: usize = LEDS_SECOND_SECTION * 12;

pub(crate) struct SpiConfig<'a, T: Instance> {
    pub spim: Peri<'a, T>,
    pub sck: Peri<'a, AnyPin>,
    pub led_pin: Peri<'a, AnyPin>,
}

#[embassy_executor::task]
pub async fn led_task(
    rng: Peri<'static, RNG>, spi_config_1: SpiConfig<'static, SPI2>,
    spi_config_2: SpiConfig<'static, SPI3>, analog_default_value: u16, analog_maximum_value: u16,
) {
    info!("Starting LED task...");

    let mut config = Config::default();
    config.frequency = Frequency::M4;

    let spi_1 = Spim::new_txonly(
        spi_config_1.spim,
        Irqs,
        spi_config_1.sck,
        spi_config_1.led_pin,
        config.clone(),
    );

    let mut buffer_1 = [0; FIRST_SECTION_BUFFERS_SIZE];
    let mut ws2812_1 = Ws2812::new(spi_1, &mut buffer_1);

    let spi_2 = Spim::new_txonly(
        spi_config_2.spim,
        Irqs,
        spi_config_2.sck,
        spi_config_2.led_pin,
        config,
    );

    let mut buffer_2 = [0; SECOND_SECTION_BUFFERS_SIZE];
    let mut ws2812_2 = Ws2812::new(spi_2, &mut buffer_2);

    // Setup Pseudo Random Number Generator
    let mut prng = setup_prng(rng).await;

    let mut data = create_data();

    let mut active_kind = AnimationKind::MultiColorStrand;
    let mut active_animation = Animation::new(active_kind, &mut prng);

    info!("Creating default animation settings");
    let mut settings = Settings::new(
        COLORS_INDEX_DEFAULT,
        analog_default_value,
        analog_default_value,
        analog_maximum_value,
        COLORS_TOTAL,
    );

    let mut delay = Delay;

    loop {
        if let Some(()) = ANIMATION_CHANGED_SIGNAL.try_take() {
            info!("Animation changed signal received");
            active_kind = active_kind.next();
            active_animation = Animation::new(active_kind, &mut prng);
            reset_data(&mut data);
        }

        if let Some(brightness) = BRIGHTNESS_READ_SIGNAL.try_take() {
            settings.set_brightness(brightness);
        }

        if let Some(()) = COLOR_CHANGED_SIGNAL.try_take() {
            info!("Color changed signal received");
            settings.set_color_index((settings.color_index() + 1) % COLORS_TOTAL);
        }

        if let Some(delay) = DELAY_READ_SIGNAL.try_take() {
            settings.set_delay(delay);
        }

        debug!("Updating animation data");
        active_animation.update(&mut data, &settings);

        debug!("Rendering animation");
        active_animation
            .render(&data, &mut ws2812_1, &mut ws2812_2, &mut delay, &settings)
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
