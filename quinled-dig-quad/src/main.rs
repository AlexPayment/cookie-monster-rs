#![no_std]
#![no_main]

use embassy_executor::Spawner;
use {esp_backtrace as _, esp_println as _};

#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) {}
