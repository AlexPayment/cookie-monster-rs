use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;

pub static ANIMATION_CHANGED_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();
pub static BRIGHTNESS_READ_SIGNAL: Signal<CriticalSectionRawMutex, u16> = Signal::new();
pub static COLOR_CHANGED_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();
pub static DELAY_READ_SIGNAL: Signal<CriticalSectionRawMutex, u16> = Signal::new();
