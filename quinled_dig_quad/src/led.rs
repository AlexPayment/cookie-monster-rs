use cookie_monster_common::animations::{
    Animation, AnimationKind, COLORS_INDEX_DEFAULT, COLORS_TOTAL, LEDS_FIRST_SECTION,
    LEDS_SECOND_SECTION, Settings, create_data, reset_data,
};
use cookie_monster_common::signal::{
    ANIMATION_CHANGED_SIGNAL, BRIGHTNESS_READ_SIGNAL, COLOR_CHANGED_SIGNAL, DELAY_READ_SIGNAL,
};
use defmt::{debug, info};
use embassy_time::Delay;
use esp_hal::dma::{AnySpiDmaChannel, DmaRxBuf, DmaTxBuf};
use esp_hal::gpio::AnyPin;
use esp_hal::rng::Rng;
use esp_hal::spi::master::{AnySpi, Config, Spi, SpiDmaBus};
use esp_hal::time::Rate;
use esp_hal::{Blocking, dma_buffers};
use rand::SeedableRng;
use rand::rngs::SmallRng;
use ws2812_spi::prerendered::Ws2812;

// 12 is calculated by knowing that we're using 8 bits per color (3 bytes total), and that
// ws2812_spi converts each color byte to 4 SPI bytes.
const FIRST_SECTION_BUFFERS_SIZE: usize = LEDS_FIRST_SECTION * 12;

// 12 is calculated by knowing that we're using 8 bits per color (3 bytes total), and that
// ws2812_spi converts each color byte to 4 SPI bytes.
const SECOND_SECTION_BUFFERS_SIZE: usize = LEDS_SECOND_SECTION * 12;

// According to the ws2812_spi documentation, the SPI frequency must be between 2 and 3.8 MHz.
// Though, in practice, it seems that the lower limit is really around 2.2 MHz on this board.
const SPI_FREQUENCY: Rate = Rate::from_khz(3_800);

pub(crate) struct SpiConfig<'a> {
    pub spi: AnySpi<'a>,
    pub dma_channel: AnySpiDmaChannel<'a>,
    pub led_pin: AnyPin<'a>,
}

#[embassy_executor::task]
pub async fn led_task(
    spi_config_1: SpiConfig<'static>, spi_config_2: SpiConfig<'static>, analog_default_value: u16,
    analog_maximum_value: u16,
) {
    info!("Starting LED task...");

    let (dma_rx_buffer_1, dma_tx_buffer_1, dma_rx_buffer_2, dma_tx_buffer_2) = create_dma_buffers();

    let mut buffer_1 = [0; FIRST_SECTION_BUFFERS_SIZE];
    let mut ws2812_1 = create_ws2812_driver(
        spi_config_1,
        dma_rx_buffer_1,
        dma_tx_buffer_1,
        &mut buffer_1,
    );

    let mut buffer_2 = [0; SECOND_SECTION_BUFFERS_SIZE];
    let mut ws2812_2 = create_ws2812_driver(
        spi_config_2,
        dma_rx_buffer_2,
        dma_tx_buffer_2,
        &mut buffer_2,
    );

    // Setup Pseudo Random Number Generator
    let rng = Rng::new();
    let mut prng = SmallRng::seed_from_u64(u64::from(rng.random()));

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

fn create_dma_buffers() -> (DmaRxBuf, DmaTxBuf, DmaRxBuf, DmaTxBuf) {
    debug!("Creating DMA buffers");

    // 4 is the smallest size allowed due to the required byte alignment. This is true even if the
    // buffer is unused.
    let (rx_buffer, rx_descriptors, tx_buffer, tx_descriptors) =
        dma_buffers!(4, FIRST_SECTION_BUFFERS_SIZE);
    let dma_rx_buffer_1 = DmaRxBuf::new(rx_descriptors, rx_buffer).unwrap();
    let dma_tx_buffer_1 = DmaTxBuf::new(tx_descriptors, tx_buffer).unwrap();

    let (rx_buffer, rx_descriptors, tx_buffer, tx_descriptors) =
        dma_buffers!(4, SECOND_SECTION_BUFFERS_SIZE);
    let dma_rx_buffer_2 = DmaRxBuf::new(rx_descriptors, rx_buffer).unwrap();
    let dma_tx_buffer_2 = DmaTxBuf::new(tx_descriptors, tx_buffer).unwrap();

    (
        dma_rx_buffer_1,
        dma_tx_buffer_1,
        dma_rx_buffer_2,
        dma_tx_buffer_2,
    )
}

fn create_ws2812_driver<'a>(
    spi_config: SpiConfig<'a>, dma_rx_buffer: DmaRxBuf, dma_tx_buffer: DmaTxBuf,
    buffer: &'a mut [u8],
) -> Ws2812<'a, SpiDmaBus<'a, Blocking>> {
    let spi = Spi::new(
        spi_config.spi,
        Config::default().with_frequency(SPI_FREQUENCY),
    )
    .unwrap()
    .with_mosi(spi_config.led_pin)
    .with_dma(spi_config.dma_channel)
    .with_buffers(dma_rx_buffer, dma_tx_buffer);

    Ws2812::new(spi, buffer)
}
