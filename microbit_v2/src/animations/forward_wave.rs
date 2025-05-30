use crate::animations::{Animation, ForwardWave};
use cookie_monster_common::animations;
use cookie_monster_common::animations::{COLORS, LedData, NUM_LEDS, Settings};
use embedded_hal::delay::DelayNs;
use microbit::hal::Timer;
use microbit::hal::spi::Spi;
use microbit::pac::{SPI0, TIMER0};
use smart_leds_trait::SmartLedsWrite;
use ws2812_spi::Ws2812;

const WAVE_LENGTH: usize = 15;
const WAVE_SECTION_LENGTH: usize = WAVE_LENGTH / 5;

impl<'a> ForwardWave<'a> {
    pub(crate) fn new(data: &'a LedData) -> Self {
        Self {
            data,
            position: 0,
            wrapped: false,
        }
    }

    fn get_wave(&self, settings: &Settings) -> [f32; WAVE_LENGTH] {
        let mut wave = [0.0; WAVE_LENGTH];

        wave[0..WAVE_SECTION_LENGTH].iter_mut().for_each(|item| {
            *item = self.brightness(settings) / 10.0;
        });
        wave[WAVE_SECTION_LENGTH..(2 * WAVE_SECTION_LENGTH)]
            .iter_mut()
            .for_each(|item| {
                *item = self.brightness(settings);
            });
        wave[(2 * WAVE_SECTION_LENGTH)..(3 * WAVE_SECTION_LENGTH)]
            .iter_mut()
            .for_each(|item| {
                *item = self.brightness(settings) / 4.0;
            });
        wave[(3 * WAVE_SECTION_LENGTH)..(4 * WAVE_SECTION_LENGTH)]
            .iter_mut()
            .for_each(|item| {
                *item = self.brightness(settings) / 6.0;
            });
        wave[(4 * WAVE_SECTION_LENGTH)..WAVE_LENGTH]
            .iter_mut()
            .for_each(|item| {
                *item = self.brightness(settings) / 10.0;
            });

        wave
    }
}

impl Animation for ForwardWave<'_> {
    fn brightness(&self, settings: &Settings) -> f32 {
        settings.brightness()
    }

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
                        animations::create_color_with_brightness(
                            COLORS[settings.color_index()],
                            *item,
                        );
                } else {
                    self.data.borrow_mut()[led_index as usize] =
                        animations::create_color_with_brightness(
                            COLORS[settings.color_index()],
                            *item,
                        );
                }
            } else if led_index >= 0 {
                self.data.borrow_mut()[led_index as usize] =
                    animations::create_color_with_brightness(COLORS[settings.color_index()], *item);
            }
        }

        self.position += 1;
        if self.position >= NUM_LEDS {
            self.position = 0;
            self.wrapped = true;
        }

        ws2812.write(self.data.borrow().iter().copied()).unwrap();
        timer.delay_ms(settings.delay());
    }

    fn reset(&mut self) {
        animations::reset_data(self.data);
        self.position = 0;
        self.wrapped = false;
    }
}
