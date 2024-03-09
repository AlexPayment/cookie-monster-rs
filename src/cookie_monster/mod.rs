use microbit::display::blocking::Display;
use microbit::gpio::DisplayPins;
use microbit::hal::Timer;
use microbit::pac::TIMER0;
use rtt_target::debug_rprintln;

#[derive(Clone, Copy)]
enum BrightnessLevel {
    Low = 10,
    LowMedium = 25,
    Medium = 50,
    MediumHigh = 75,
    High = 100,
}

pub(crate) struct CookieMonster {
    brightness: BrightnessLevel,
    display: Display,
    timer: Timer<TIMER0>,
}

impl CookieMonster {
    pub fn new(display_pins: DisplayPins, timer0: TIMER0) -> Self {
        let mut cookie_monster = CookieMonster {
            brightness: BrightnessLevel::Medium,
            display: Display::new(display_pins),
            timer: Timer::new(timer0),
        };
        cookie_monster.update_display();
        cookie_monster
    }

    pub fn cycle_brightness(&mut self) {
        self.brightness = match self.brightness {
            BrightnessLevel::Low => BrightnessLevel::LowMedium,
            BrightnessLevel::LowMedium => BrightnessLevel::Medium,
            BrightnessLevel::Medium => BrightnessLevel::MediumHigh,
            BrightnessLevel::MediumHigh => BrightnessLevel::High,
            BrightnessLevel::High => BrightnessLevel::Low,
        };

        self.update_display();
    }

    /// Updates the display with the current brightness
    fn update_display(&mut self) {
        let mut leds = [[0; 5]; 5];
        match self.brightness {
            BrightnessLevel::Low => {
                debug_rprintln!("Display low brightness");
                leds[4] = [1; 5];
            }
            BrightnessLevel::LowMedium => {
                debug_rprintln!("Display low medium brightness");
                leds[3] = [1; 5];
                leds[4] = [1; 5];
            }
            BrightnessLevel::Medium => {
                debug_rprintln!("Display medium brightness");
                leds[2] = [1; 5];
                leds[3] = [1; 5];
                leds[4] = [1; 5];
            }
            BrightnessLevel::MediumHigh => {
                debug_rprintln!("Display medium high brightness");
                leds[1] = [1; 5];
                leds[2] = [1; 5];
                leds[3] = [1; 5];
                leds[4] = [1; 5];
            }
            BrightnessLevel::High => {
                debug_rprintln!("Display high brightness");
                leds = [[1; 5]; 5];
            }
        }
        self.display.show(&mut self.timer, leds, 1000);
    }
}
