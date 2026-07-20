use crate::animations;
use crate::animations::{COLORS, LedData, Settings, VERTICAL_SLICES};

pub struct UniColorFrontToBackWave {
    position: usize,
}

impl UniColorFrontToBackWave {
    pub(crate) const BRIGHTNESS_DAMPING_FACTOR: f32 = 1.0;

    pub(crate) fn new() -> Self {
        Self { position: 0 }
    }

    pub(crate) fn update(&mut self, data: &mut LedData, settings: &Settings) {
        animations::reset_data(data);

        let slice = &VERTICAL_SLICES[self.position];

        for led in slice {
            led.map(|l| {
                data[usize::from(l)] = COLORS[settings.color_index()];
            });
        }

        self.position = (self.position + 1) % VERTICAL_SLICES.len();
    }
}
