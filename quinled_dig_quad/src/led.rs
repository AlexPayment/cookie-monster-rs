use cookie_monster_common::animations::{
    Animation, AnimationKind, DEFAULT_COLOR_INDEX, NUM_COLORS, NUM_LEDS, Settings, create_data,
};
use cookie_monster_common::signal::{
    ANIMATION_CHANGED_SIGNAL, BRIGHTNESS_READ_SIGNAL, COLOR_CHANGED_SIGNAL, DELAY_READ_SIGNAL,
};
use defmt::{debug, info};
use embassy_time::Delay;
use esp_hal::dma::{AnySpiDmaChannel, DmaRxBuf, DmaTxBuf};
use esp_hal::dma_buffers;
use esp_hal::gpio::AnyPin;
use esp_hal::rng::Rng;
use esp_hal::spi::master::{AnySpi, Config as SpiConfig, Spi};
use esp_hal::time::Rate;
use rand::SeedableRng;
use rand::rngs::SmallRng;
use ws2812_spi::prerendered::Ws2812;

// According to the ws2812_spi documentation, the SPI frequency must be between 2 and 3.8 MHz.
// Though, in practice, it seems that the lower limit is really around 2.2 MHz on this board.
const SPI_FREQUENCY: Rate = Rate::from_khz(3_800);

#[embassy_executor::task]
pub async fn led_task(
    spi: AnySpi<'static>, dma_channel: AnySpiDmaChannel<'static>, led: AnyPin<'static>,
    analog_default_value: u16, analog_maximum_value: u16,
) {
    info!("Starting LED task...");

    #[allow(clippy::manual_div_ceil)]
    let (rx_buffer, rx_descriptors, tx_buffer, tx_descriptors) = dma_buffers!(NUM_LEDS * 12);
    let dma_rx_buf = DmaRxBuf::new(rx_descriptors, rx_buffer).unwrap();
    let dma_tx_buf = DmaTxBuf::new(tx_descriptors, tx_buffer).unwrap();

    let spi = Spi::new(spi, SpiConfig::default().with_frequency(SPI_FREQUENCY))
        .unwrap()
        .with_mosi(led)
        .with_dma(dma_channel)
        .with_buffers(dma_rx_buf, dma_tx_buf);

    let mut buffer = [0; NUM_LEDS * 12];
    let mut ws2812 = Ws2812::new(spi, &mut buffer);

    // Setup Pseudo Random Number Generator
    let rng = Rng::new();
    let mut prng = SmallRng::seed_from_u64(u64::from(rng.random()));

    let data = create_data();

    let mut active_kind = AnimationKind::MultiColorStrand;
    let mut active_animation = Animation::new(active_kind, &data, &mut prng);

    info!("Creating default animation settings");
    let mut settings = Settings::new(
        DEFAULT_COLOR_INDEX,
        analog_default_value,
        analog_default_value,
        analog_maximum_value,
        NUM_COLORS,
    );

    let mut delay = Delay;

    loop {
        if let Some(()) = ANIMATION_CHANGED_SIGNAL.try_take() {
            info!("Animation changed signal received");
            active_kind = active_kind.next();
            active_animation = Animation::new(active_kind, &data, &mut prng);
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
        active_animation.update(&settings);

        debug!("Rendering animation");
        active_animation
            .render(&mut ws2812, &mut delay, &settings)
            .await;
    }
}
