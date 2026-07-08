use cookie_monster_common::animations::{
    DEFAULT_COLOR_INDEX, NUM_ANIMATIONS, NUM_COLORS, NUM_LEDS, Settings, create_data,
    initialize_animations,
};
use cookie_monster_common::signal::{
    ANIMATION_CHANGED_SIGNAL, BRIGHTNESS_READ_SIGNAL, COLOR_CHANGED_SIGNAL, DELAY_READ_SIGNAL,
};
use core::mem::MaybeUninit;
use defmt::{debug, info};
use embassy_time::Delay;
use esp_hal::clock::Clocks;
use esp_hal::gpio::{AnyPin, Level};
use esp_hal::peripherals::RMT;
use esp_hal::rmt::{PulseCode, Rmt, TxChannelConfig, TxChannelCreator, TxTransaction};
use esp_hal::rng::Rng;
use esp_hal::time::Rate;
use rand::SeedableRng;
use rand::rngs::SmallRng;
use rgb::RGB8;
use smart_leds_trait::SmartLedsWrite;

const BUFFER_SIZE: usize = NUM_LEDS * 24 + 1;
static mut PULSE_BUFFER: [MaybeUninit<PulseCode>; BUFFER_SIZE] =
    [MaybeUninit::uninit(); BUFFER_SIZE];

pub struct Ws2812Rmt<'d, 'a> {
    channel: Option<esp_hal::rmt::Channel<'d, esp_hal::Blocking, esp_hal::rmt::Tx>>,
    transaction: Option<TxTransaction<'d, 'static>>,
    pulses: (PulseCode, PulseCode),
    buffer: &'a mut [MaybeUninit<PulseCode>],
}

impl<'d, 'a> Ws2812Rmt<'d, 'a> {
    pub fn new<Ch, P>(channel: Ch, pin: P, buffer: &'a mut [MaybeUninit<PulseCode>]) -> Self
    where
        Ch: TxChannelCreator<'d, esp_hal::Blocking>,
        P: esp_hal::gpio::interconnect::PeripheralOutput<'d>,
    {
        let config = TxChannelConfig::default()
            .with_clk_divider(1)
            .with_idle_output_level(Level::Low)
            .with_memsize(1)
            .with_carrier_modulation(false)
            .with_idle_output(true);

        let channel = channel.configure_tx(&config).unwrap().with_pin(pin);

        let clocks = Clocks::get();
        let src_clock = clocks.apb_clock.as_hz() / 1_000_000;

        let zero = PulseCode::new(
            Level::High,
            ((350 * src_clock) / 1000) as u16,
            Level::Low,
            ((900 * src_clock) / 1000) as u16,
        );
        let one = PulseCode::new(
            Level::High,
            ((900 * src_clock) / 1000) as u16,
            Level::Low,
            ((350 * src_clock) / 1000) as u16,
        );

        Self {
            channel: Some(channel),
            transaction: None,
            pulses: (zero, one),
            buffer,
        }
    }
}

impl<'d, 'a> SmartLedsWrite for Ws2812Rmt<'d, 'a> {
    type Error = ();
    type Color = RGB8;

    fn write<T, I>(&mut self, iterator: T) -> Result<(), <Self as SmartLedsWrite>::Error>
    where
        T: IntoIterator<Item = I>,
        I: Into<Self::Color>,
    {
        if let Some(tx) = self.transaction.take() {
            match tx.wait() {
                Ok(chan) => {
                    self.channel = Some(chan);
                }
                Err((_err, chan)) => {
                    self.channel = Some(chan);
                    return Err(());
                }
            }
        }

        let mut index = 0;
        let zero = self.pulses.0;
        let one = self.pulses.1;

        for item in iterator {
            let color: RGB8 = item.into();
            let g = color.g;
            let r = color.r;
            let b = color.b;

            for &val in &[g, r, b] {
                for bit in (0..8).rev() {
                    let is_one = (val & (1 << bit)) != 0;
                    if index < self.buffer.len() - 1 {
                        self.buffer[index].write(if is_one { one } else { zero });
                        index += 1;
                    } else {
                        return Err(());
                    }
                }
            }
        }
        if index < self.buffer.len() {
            self.buffer[index].write(PulseCode::end_marker());
        } else {
            return Err(());
        }

        let slice: &[PulseCode] = unsafe {
            core::slice::from_raw_parts(self.buffer.as_ptr() as *const PulseCode, index + 1)
        };

        let channel = self.channel.take().unwrap();
        match channel.transmit(slice) {
            Ok(transaction) => {
                self.transaction = Some(transaction);
                Ok(())
            }
            Err((_err, chan)) => {
                self.channel = Some(chan);
                Err(())
            }
        }
    }
}

static mut LED_DRIVER: Option<Ws2812Rmt<'static, 'static>> = None;

#[embassy_executor::task]
pub async fn led_task(
    _spi: esp_hal::spi::master::AnySpi<'static>,
    _dma_channel: esp_hal::dma::AnySpiDmaChannel<'static>, rmt: RMT<'static>,
    led1: AnyPin<'static>, analog_default_value: u16, analog_maximum_value: u16,
) {
    info!("Starting LED task...");

    info!("Configuring RMT");
    let rmt =
        Rmt::new(rmt, Rate::from_mhz(80)).expect("Configuring RMT at its maximum frequency 80 MHz");

    debug!("Configuring RMT Smart LED 1");
    let led1 = unsafe {
        let ptr = &raw mut LED_DRIVER;
        let buf_ref = &mut *(&raw mut PULSE_BUFFER);
        *ptr = Some(Ws2812Rmt::new(rmt.channel0, led1, buf_ref));
        (*ptr).as_mut().unwrap()
    };
    debug!("RMT Smart LED 1 configured successfully!");

    // Setup Pseudo Random Number Generator
    debug!("Initializing hardware RNG...");
    let rng = Rng::new();
    debug!("Hardware RNG initialized! Getting random seed...");
    let random_value = rng.random();
    debug!("Random seed received: {}", random_value);
    let mut prng = SmallRng::seed_from_u64(u64::from(random_value));

    debug!("Creating LED data");
    let data = create_data();

    let mut animation_index = 0;
    let mut animations = initialize_animations(&data, &mut prng);

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
            .render(led1, &mut delay, &settings)
            .await;
    }
}
