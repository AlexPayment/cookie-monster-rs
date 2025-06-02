use crate::animations::carrousel::Carrousel;
use crate::animations::double_carrousel::DoubleCarrousel;
use core::cell::RefCell;
use core::cmp;
use defmt::{Format, info};
use embedded_hal::delay::DelayNs;
use embedded_hal::spi;
use rand::Rng;
use rand::rngs::SmallRng;
use smart_leds::RGB8;
use smart_leds::colors::{
    BLUE, DARK_GREEN, DARK_RED, DARK_TURQUOISE, GOLD, GREEN, INDIGO, MIDNIGHT_BLUE, PURPLE, RED,
    WHITE,
};
use smart_leds_trait::SmartLedsWrite;

pub const COLORS: [RGB8; NUM_COLORS] = [
    WHITE,
    RED,
    DARK_RED,
    GOLD,
    GREEN,
    DARK_GREEN,
    DARK_TURQUOISE,
    BLUE,
    MIDNIGHT_BLUE,
    PURPLE,
    INDIGO,
];
pub const DEFAULT_COLOR_INDEX: usize = 9;
pub const NUM_ANIMATIONS: usize = 2;
pub const NUM_COLORS: usize = 11;
pub const NUM_LEDS: usize = 96 * 10;

pub type LedData = RefCell<[RGB8; NUM_LEDS]>;

pub enum Animation<'a> {
    Carrousel(Carrousel<'a>),
    DoubleCarrousel(DoubleCarrousel<'a>),
}

impl Animation<'_> {
    /// Renders the animation.
    pub fn render(
        &mut self, ws2812: &mut impl SmartLedsWrite<Color = RGB8, Error = impl spi::Error>,
        delay: &mut impl DelayNs, settings: &Settings,
    ) {
        match self {
            Animation::Carrousel(carrousel) => carrousel.render(ws2812, delay, settings),
            Animation::DoubleCarrousel(double_carrousel) => {
                double_carrousel.render(ws2812, delay, settings)
            }
        }
    }

    /// Resets the animation to its initial state.
    pub fn reset(&mut self) {
        match self {
            Animation::Carrousel(carrousel) => carrousel.reset(),
            Animation::DoubleCarrousel(double_carrousel) => double_carrousel.reset(),
        }
    }

    /// Updates the state of the animation based on the settings.
    pub fn update(&mut self, settings: &Settings) {
        match self {
            Animation::Carrousel(carrousel) => carrousel.update(settings),
            Animation::DoubleCarrousel(double_carrousel) => double_carrousel.update(settings),
        }
    }
}

/// Common settings for the animations.
#[derive(Clone, Copy, Debug, Format)]
pub struct Settings {
    /// Brightness of the LEDs, between 0.0 and 1.0.
    brightness: f32,

    /// Index of the color to be used in the animation.
    ///
    /// Multicolor animations generally ignore this value.
    color_index: usize,

    /// Delay between frames in milliseconds.
    delay: u32,

    /// Maximum value of the analog sensors (potentiometers).
    max_analog_value: u16,

    /// Number of colors available for the animations.
    num_colors: usize,
}

impl Settings {
    #[must_use]
    pub fn new(
        color_index: usize, brightness: u16, delay: u16, max_analog_value: u16, num_colors: usize,
    ) -> Self {
        Self {
            brightness: calculate_brightness(brightness, max_analog_value),
            color_index,
            delay: calculate_delay(delay, max_analog_value),
            max_analog_value,
            num_colors,
        }
    }

    #[must_use]
    pub fn brightness(&self) -> f32 {
        self.brightness
    }

    #[must_use]
    pub fn color_index(&self) -> usize {
        self.color_index
    }

    #[must_use]
    pub fn delay(&self) -> u32 {
        self.delay
    }

    /// Increment the color index and wrap around if it exceeds the number of colors.
    pub fn increment_color_index(&mut self) {
        self.color_index = (self.color_index + 1) % self.num_colors;
    }

    pub fn set_brightness(&mut self, brightness: u16) {
        self.brightness = calculate_brightness(brightness, self.max_analog_value);
    }

    pub fn set_color_index(&mut self, color_index: usize) {
        self.color_index = color_index;
    }

    pub fn set_delay(&mut self, delay: u16) {
        self.delay = calculate_delay(delay, self.max_analog_value);
    }
}

pub fn calculate_index(value: u16, max_value: u16, num_values: usize) -> usize {
    let index = (f32::from(value) / f32::from(max_value) * num_values as f32) as usize;
    cmp::min(index, num_values - 1)
}

/// Create a color from a provided color adjusted by a brightness value between 0 and 1.
pub fn create_color_with_brightness(color: RGB8, brightness: f32) -> RGB8 {
    RGB8::new(
        (f32::from(color.r) * brightness) as u8,
        (f32::from(color.g) * brightness) as u8,
        (f32::from(color.b) * brightness) as u8,
    )
}

/// Create a new [`LedData`] structure initialized with default colors.
pub fn create_data() -> LedData {
    RefCell::new([RGB8::default(); NUM_LEDS])
}

/// Initialize the animations with the provided LED data and a pseudo-random number generator.
pub fn initialize_animations<'a>(
    led_data: &'a LedData, prng: &mut SmallRng,
) -> [Animation<'a>; NUM_ANIMATIONS] {
    info!("Initialize animations...");
    let carrousel = Carrousel::new(led_data, prng.random());
    let double_carrousel = DoubleCarrousel::new(led_data, prng.random());

    [
        Animation::Carrousel(carrousel),
        Animation::DoubleCarrousel(double_carrousel),
    ]
}

/// Resets the LEDs data to its default state.
pub fn reset_data(data: &LedData) {
    let mut data = data.borrow_mut();
    for i in 0..NUM_LEDS {
        data[i] = RGB8::default();
    }
}

/// Calculate the brightness based on the value of the potentiometer reading.
///
/// The value is between 0 and 1.
fn calculate_brightness(value: u16, max_value: u16) -> f32 {
    f32::from(value) / f32::from(max_value)
}

/// Calculate the delay in milliseconds based on the value of the potentiometer reading.
///
/// The delay is calculated as a fraction of the maximum analog value times one thousand. The
/// resulting value is then clamped to a minimum of 1.
fn calculate_delay(value: u16, max_value: u16) -> u32 {
    cmp::max((f32::from(value) / f32::from(max_value) * 1000.0) as u32, 1)
}

pub mod carrousel;
pub mod double_carrousel;
