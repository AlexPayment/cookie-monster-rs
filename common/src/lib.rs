#![no_std]

use core::time::Duration;

pub trait Timer {
    fn pause(&mut self, duration: Duration);

    #[allow(async_fn_in_trait)]
    async fn pause_async(&self, duration: Duration);
}

pub mod animations;
