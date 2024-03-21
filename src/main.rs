#![no_std]
#![no_main]

use core::cell::RefCell;
use core::cmp;
use cortex_m::interrupt::{free, Mutex};
use cortex_m_rt::entry;
use effects::{Effect, ForwardWave, MultiColorSparkle, Settings, UniColorSparkle, NUM_COLORS, NUM_LEDS};
use microbit::hal::gpio::p0::{Parts, P0_14, P0_23};
use microbit::hal::gpio::{Floating, Input, Level};
use microbit::hal::gpiote::Gpiote;
use microbit::hal::prelude::_embedded_hal_adc_OneShot;
use microbit::hal::saadc::{Resolution, SaadcConfig};
use microbit::hal::{spi, Saadc, Timer};
use microbit::pac::{interrupt, GPIOTE};
use microbit::{hal, pac, Peripherals};
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use smart_leds::RGB8;
use ws2812_spi::Ws2812;

mod effects;

static COLOR: Mutex<RefCell<usize>> = Mutex::new(RefCell::new(9));
static EFFECT: Mutex<RefCell<usize>> = Mutex::new(RefCell::new(0));
static GPIO: Mutex<RefCell<Option<Gpiote>>> = Mutex::new(RefCell::new(None));

const NUM_EFFECTS: usize = 3;

#[entry]
fn main() -> ! {
    rtt_init_print!();

    // Setup all peripherals and the WS2812 device
    let peripherals = Peripherals::take().unwrap();

    let port0 = Parts::new(peripherals.P0);
    let sck = port0.p0_17.into_push_pull_output(Level::Low).degrade();
    // The SPI MOSI pin is pin 15 on the micro:bit.
    let mosi = port0.p0_13.into_push_pull_output(Level::Low).degrade();
    let miso = port0.p0_01.into_floating_input().degrade();
    let pins = spi::Pins {
        sck,
        miso: Some(miso),
        mosi: Some(mosi),
    };
    let spi = spi::Spi::new(peripherals.SPI0, pins, spi::Frequency::M4, spi::MODE_0);
    let mut ws2812 = Ws2812::new(spi);
    let mut timer = Timer::new(peripherals.TIMER0);

    let saadc_config = SaadcConfig {
        resolution: Resolution::_12BIT,
        ..Default::default()
    };
    let mut adc = Saadc::new(peripherals.SAADC, saadc_config);
    // This analog pin is the big 0 connector on the micro:bit.
    let mut brightness_pin = port0.p0_02.into_floating_input();
    let mut delay_pin = port0.p0_03.into_floating_input();
    // This analog pin is the big 2 connector on the micro:bit.
    let mut color_pin = port0.p0_04.into_floating_input();

    // Setup Pseudo Random Number Generator
    let mut rng = hal::Rng::new(peripherals.RNG);

    init_buttons(
        peripherals.GPIOTE,
        port0.p0_14.into_floating_input(),
        port0.p0_23.into_floating_input(),
    );

    // Get the maximum value of the potentiometer. Must match the resolution of the ADC which is set to 12 bits above.
    let max_value = 2u32.pow(12) - 1;
    rprintln!("Max potentiometer value: {}", max_value);

    let data = RefCell::new([RGB8::default(); NUM_LEDS]);

    rprintln!("Creating effects...");
    let mut uni_color_sparkle = UniColorSparkle::new(&data, rng.random_u64());
    let mut multi_color_sparkle = MultiColorSparkle::new(&data, rng.random_u64());
    let mut forward_wave = ForwardWave::new(&data);

    let effect: [&mut dyn Effect; NUM_EFFECTS] = [&mut uni_color_sparkle, &mut multi_color_sparkle, &mut forward_wave];

    rprintln!("Starting main loop...");
    loop {
        let brightness = cmp::max(1, adc.read(&mut brightness_pin).unwrap_or((max_value / 2) as i16));
        let delay = cmp::max(2, adc.read(&mut delay_pin).unwrap_or((max_value / 2) as i16));
        let color = adc.read(&mut color_pin).unwrap_or((max_value / 2) as i16);

        rprintln!("Brightness: {}, Delay: {}, Color: {}", brightness, delay, color);

        // let converted_color = convert_color(color);
        // rprintln!("Converted color: {:?}", converted_color);

        let mut color_index = 0;
        let mut effect_index = 0;

        free(|cs| {
            color_index = *COLOR.borrow(cs).borrow();
            effect_index = *EFFECT.borrow(cs).borrow();
        });

        let settings = Settings::new(
            color_index,
            // Value between 0 and 1
            brightness as f32 / max_value as f32,
            // The 12-bit value is too high for a good delay, so we divide it by 2.
            (delay / 2) as u32,
        );

        rprintln!("{:?}", settings);
        rprintln!("Current effect: {}", effect_index);

        effect[effect_index].render(
            &mut ws2812,
            &mut timer,
            &settings,
        );
    }
}

/// Converts a 12-bit color value to three 8-bit color values.
fn convert_color(value: i16) -> RGB8 {
    let b: u8 = (value & 0x00f) as u8;
    let g: u8 = ((value >> 4) & 0x00f) as u8;
    let r: u8 = ((value >> 8) & 0x00f) as u8;
    RGB8 {
        r: r << 4 | r,
        g: g << 4 | g,
        b: b << 4 | b,
    }
}

fn init_buttons(
    board_gpiote: GPIOTE, button_a_pin: P0_14<Input<Floating>>,
    button_b_pin: P0_23<Input<Floating>>,
) {
    let gpiote = Gpiote::new(board_gpiote);

    let channel0 = gpiote.channel0();
    channel0
        .input_pin(&button_a_pin.degrade())
        .hi_to_lo()
        .enable_interrupt();
    channel0.reset_events();

    let channel1 = gpiote.channel1();
    channel1
        .input_pin(&button_b_pin.degrade())
        .hi_to_lo()
        .enable_interrupt();
    channel1.reset_events();

    free(move |cs| {
        *GPIO.borrow(cs).borrow_mut() = Some(gpiote);

        unsafe {
            pac::NVIC::unmask(pac::Interrupt::GPIOTE);
        }
        pac::NVIC::unpend(pac::Interrupt::GPIOTE);
    });
}

#[interrupt]
fn GPIOTE() {
    free(|cs| {
        if let Some(gpiote) = GPIO.borrow(cs).borrow().as_ref() {
            let a_pressed = gpiote.channel0().is_event_triggered();
            let b_pressed = gpiote.channel1().is_event_triggered();

            if a_pressed {
                EFFECT.borrow(cs).replace_with(|e| (*e + 1) % NUM_EFFECTS);
            } else if b_pressed {
                // Cycle color
                COLOR.borrow(cs).replace_with(|c| (*c + 1) % NUM_COLORS);
            }

            gpiote.channel0().reset_events();
            gpiote.channel1().reset_events();
        }
    });
}
