#![no_std]

use core::time::Duration;

pub trait Timer {
    fn pause(&mut self, duration: Duration);
}

pub mod animations;
