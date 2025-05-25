#![no_std]

use core::time::Duration;

pub trait Timer {
    fn pause(&self, duration: Duration);
    async fn pause_async(&self, duration: Duration);
}

pub mod animations;
