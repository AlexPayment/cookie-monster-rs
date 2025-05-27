#![no_std]

use core::time::Duration;

pub trait Timer {
    /// Starts a blocking timer for a given duration.
    fn start(&mut self, duration: Duration);
}

pub mod animations;
