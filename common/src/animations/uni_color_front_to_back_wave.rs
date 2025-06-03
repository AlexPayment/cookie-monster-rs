use crate::animations;
use crate::animations::{COLORS, LedData, Settings, VERTICAL_SLICES};
use embedded_hal::spi::Error as SpiError;
use embedded_hal_async::delay::DelayNs;
use smart_leds::RGB8;
use smart_leds_trait::SmartLedsWrite;

pub struct UniColorFrontToBackWave<'a> {
    data: &'a LedData,
    position: usize,
}

impl<'a> UniColorFrontToBackWave<'a> {
    pub(crate) fn new(data: &'a LedData) -> Self {
        Self { data, position: 0 }
    }

    pub(crate) async fn render(
        &mut self, ws2812: &mut impl SmartLedsWrite<Color = RGB8, Error = impl SpiError>,
        delay: &mut impl DelayNs, settings: &Settings,
    ) {
        ws2812.write(self.data.borrow().iter().copied()).unwrap();
        delay.delay_ms(settings.delay()).await;
    }

    pub(crate) fn reset(&mut self) {
        animations::reset_data(self.data);
        self.position = 0;
    }

    pub(crate) fn update(&mut self, settings: &Settings) {
        animations::reset_data(self.data);

        let slice = VERTICAL_SLICES[self.position];

        for led in &slice {
            led.map(|l| {
                self.data.borrow_mut()[l as usize] = animations::create_color_with_brightness(
                    COLORS[settings.color_index()],
                    self.brightness(settings),
                );
            });
        }

        self.position = (self.position + 1) % VERTICAL_SLICES.len();
    }

    fn brightness(&self, settings: &Settings) -> f32 {
        settings.brightness()
    }
}
