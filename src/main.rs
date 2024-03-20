#![no_std]
#![no_main]

use core::cell::RefCell;
use cortex_m::interrupt::{free, Mutex};
use cortex_m_rt::entry;
use effects::{Effect, ForwardWave, Settings, Speed, UniColorSparkle, NUM_LEDS, SETTINGS};
use microbit::hal::gpio::p0::{Parts, P0_14, P0_23};
use microbit::hal::gpio::{Floating, Input, Level};
use microbit::hal::gpiote::Gpiote;
use microbit::hal::{spi, Timer};
use microbit::pac::{interrupt, GPIOTE};
use microbit::{hal, pac, Peripherals};
use panic_rtt_target as _;
use rtt_target::rtt_init_print;
use smart_leds::RGB8;
use ws2812_spi::Ws2812;

mod cookie_monster;
mod effects;

static GPIO: Mutex<RefCell<Option<Gpiote>>> = Mutex::new(RefCell::new(None));

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
    let mut delay = Timer::new(peripherals.TIMER0);

    // Setup Pseudo Random Number Generator
    let mut rng = hal::Rng::new(peripherals.RNG);

    init_buttons(
        peripherals.GPIOTE,
        port0.p0_14.into_floating_input(),
        port0.p0_23.into_floating_input(),
    );

    let data = RefCell::new([RGB8::default(); NUM_LEDS]);
    let settings = Settings::new(RGB8::new(0x00, 0x00, 0xff), Speed::SLOW);
    free(|cs| {
        SETTINGS.borrow(cs).replace(Some(settings));
    });

    let mut uni_color_sparkle = UniColorSparkle::new(&data, rng.random_u64());
    let mut forward_wave = ForwardWave::new(&data);

    let effect: [&mut dyn Effect; 2] = [&mut uni_color_sparkle, &mut forward_wave];

    loop {
        effect[0].render(&mut ws2812, &mut delay);
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

            // TODO: Implement button press handling
            if a_pressed {
                // Cycle brightness
                SETTINGS
                    .borrow(cs)
                    .borrow_mut()
                    .as_mut()
                    .unwrap()
                    .cycle_brightness();
            } else if b_pressed {
                // Cycle effect
            }

            gpiote.channel0().reset_events();
            gpiote.channel1().reset_events();
        }
    });
}
