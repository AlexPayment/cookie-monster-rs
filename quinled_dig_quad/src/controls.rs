use defmt::{debug, info};
use embassy_time::{Duration, Timer};
use esp_hal::gpio::Pull::Down;
use esp_hal::gpio::{AnyPin, Input, InputConfig};

/// Task that waits for a button to be pressed to change the animation.
#[embassy_executor::task]
pub async fn animation_button_task(button: AnyPin, mut animation: usize, num_animations: usize) {
    let mut button = Input::new(button, InputConfig::default().with_pull(Down));

    loop {
        perform_when_button_pressed(&mut button, || {
            // Increment the animation index or wrap around if it exceeds the number of animations.
            animation = (animation + 1) % num_animations;
            info!("Animation changed to: {}", animation);
        })
        .await;
    }
}

/// Task that waits for a button to be pressed to change the color.
#[embassy_executor::task]
pub async fn color_button_task(button: AnyPin, mut color: usize, num_colors: usize) {
    let mut button = Input::new(button, InputConfig::default().with_pull(Down));

    loop {
        perform_when_button_pressed(&mut button, || {
            // Increment the color index or wrap around if it exceeds the number of colors.
            color = (color + 1) % num_colors;
            info!("Color changed to: {}", color);
        })
        .await;
    }
}

/// Executes the provided action when the button is pressed.
///
/// It handles debouncing to ensure that the action is only executed once per press.
async fn perform_when_button_pressed(button: &mut Input<'_>, action: impl FnOnce()) {
    button.wait_for_falling_edge().await;

    // Wait for a short debounce period. This allows the physical bouncing to settle. Adjust the
    // duration (e.g., 20 ms, 50 ms, 100 ms) based on the button's characteristics.
    Timer::after(Duration::from_millis(50)).await;

    // After the debounce time, check the *actual* state of the pin. If it's still low, it's a
    // valid press.
    if button.is_low() {
        // Valid button press detected.
        debug!("Button pressed!");
        // Perform the action.
        action();

        // Now, wait for the button to be released to prevent multiple triggers if the button is
        // held down, and also to allow for the next press.
        button.wait_for_rising_edge().await;

        // Add a small delay after release to debounce the release too.
        Timer::after(Duration::from_millis(50)).await;
    }

    // If button_pin.is_high() here, it means it was a very short bounce that didn't settle, so we
    // simply loop and wait for the next falling edge.
}
