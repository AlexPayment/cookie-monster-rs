use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;

pub(crate) static ANIMATION_CHANGED_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();
pub(crate) static BRIGHTNESS_READ_SIGNAL: Signal<CriticalSectionRawMutex, u16> = Signal::new();
pub(crate) static COLOR_CHANGED_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();
pub(crate) static DELAY_READ_SIGNAL: Signal<CriticalSectionRawMutex, u16> = Signal::new();
