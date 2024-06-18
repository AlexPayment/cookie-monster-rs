use crate::animations;
use crate::animations::{Animation, ForwardWave, Settings, COLORS, NUM_LEDS};
use core::cell::RefCell;
use embedded_hal::delay::DelayNs;
use microbit::hal::spi::Spi;
use microbit::hal::Timer;
use microbit::pac::{SPI0, TIMER0};
use smart_leds::RGB8;
use smart_leds_trait::SmartLedsWrite;
use ws2812_spi::Ws2812;

const WAVE_LENGTH: usize = 15;
const WAVE_SECTION_LENGTH: usize = WAVE_LENGTH / 5;

impl<'a> ForwardWave<'a> {
    pub(crate) fn new(data: &'a RefCell<[RGB8; NUM_LEDS]>) -> Self {
        Self { data, position: 0, wrapped: false }
    }

    fn get_wave(&self, settings: &Settings) -> [f32; WAVE_LENGTH] {
        let mut wave = [0.0; WAVE_LENGTH];

        wave[0..WAVE_SECTION_LENGTH].iter_mut().for_each(|item| {
            *item = settings.brightness / 10.0;
        });
        wave[WAVE_SECTION_LENGTH..(2 * WAVE_SECTION_LENGTH)].iter_mut().for_each(|item| {
            *item = settings.brightness;
        });
        wave[(2 * WAVE_SECTION_LENGTH)..(3 * WAVE_SECTION_LENGTH)].iter_mut().for_each(|item| {
            *item = settings.brightness / 4.0;
        });
        wave[(3 * WAVE_SECTION_LENGTH)..(4 * WAVE_SECTION_LENGTH)].iter_mut().for_each(|item| {
            *item = settings.brightness / 6.0;
        });
        wave[(4 * WAVE_SECTION_LENGTH)..WAVE_LENGTH].iter_mut().for_each(|item| {
            *item = settings.brightness / 10.0;
        });

        wave
    }
}

impl Animation for ForwardWave<'_> {
    fn render(
        &mut self, ws2812: &mut Ws2812<Spi<SPI0>>, timer: &mut Timer<TIMER0>, settings: &Settings,
    ) {
        animations::reset_data(self.data);

        let wave = self.get_wave(settings);

        for (i, item) in wave.iter().enumerate() {
            let led_index = (self.position - i) as isize;
            if self.wrapped {
                if led_index < 0 {
                    self.data.borrow_mut()[(NUM_LEDS as isize + led_index) as usize] =
                        animations::create_color_with_brightness(&COLORS[settings.color_index], item);
                } else {
                    self.data.borrow_mut()[led_index as usize] =
                        animations::create_color_with_brightness(&COLORS[settings.color_index], item);
                }
            } else {
                if led_index >= 0 {
                    self.data.borrow_mut()[led_index as usize] =
                        animations::create_color_with_brightness(&COLORS[settings.color_index], item);
                }
            }
        }

        self.position += 1;
        if self.position >= NUM_LEDS {
            self.position = 0;
            self.wrapped = true;
        }

        ws2812.write(self.data.borrow().iter().cloned()).unwrap();
        timer.delay_ms(settings.delay);
    }

    fn reset(&mut self) {
        animations::reset_data(self.data);
        self.position = 0;
        self.wrapped = false;
    }
}
