//! Module for the physical controls of the micro:bit

use core::cell::RefCell;

use cortex_m::interrupt::{free, Mutex};
use microbit::board::Buttons;
use microbit::hal::gpiote::Gpiote;
use microbit::pac;
use microbit::pac::{GPIOTE, interrupt};

use crate::cookie_monster::CookieMonster;

static GPIO: Mutex<RefCell<Option<Gpiote>>> = Mutex::new(RefCell::new(None));
static COOKIE_MONSTER: Mutex<RefCell<Option<CookieMonster>>> = Mutex::new(RefCell::new(None));

/// Initializes the buttons and enables interrupts
pub(crate) fn init_buttons(board_gpiote: GPIOTE, board_buttons: Buttons, cookie_monster: CookieMonster) {
    let gpiote = Gpiote::new(board_gpiote);

    let channel0 = gpiote.channel0();
    channel0
        .input_pin(&board_buttons.button_a.degrade())
        .hi_to_lo()
        .enable_interrupt();
    channel0.reset_events();

    let channel1 = gpiote.channel1();
    channel1
        .input_pin(&board_buttons.button_b.degrade())
        .hi_to_lo()
        .enable_interrupt();
    channel1.reset_events();

    free(move |cs| {
        *GPIO.borrow(cs).borrow_mut() = Some(gpiote);

        unsafe {
            pac::NVIC::unmask(pac::Interrupt::GPIOTE);
        }
        pac::NVIC::unpend(pac::Interrupt::GPIOTE);

        *COOKIE_MONSTER.borrow(cs).borrow_mut() = Some(cookie_monster);
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
                COOKIE_MONSTER.borrow(cs).borrow_mut().as_mut().unwrap().cycle_brightness();
            }

            gpiote.channel0().reset_events();
            gpiote.channel1().reset_events();
        }
    });
}
