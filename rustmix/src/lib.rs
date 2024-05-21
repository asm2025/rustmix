mod app;
#[cfg(feature = "log4rs")]
pub mod log4rs;
#[cfg(feature = "slog")]
pub mod slog;
#[cfg(feature = "audio")]
pub mod sound;
#[cfg(feature = "vision")]
pub mod vision;

pub use self::app::*;
pub mod date;
pub mod error;
pub mod input;
pub mod io;
#[cfg(feature = "python")]
pub mod python;
pub mod random;
pub mod string;
pub mod threading;
pub mod vpn;
pub mod web;

use lazy_static::lazy_static;
use log::LevelFilter;
use std::sync::Mutex;

pub const LOG_DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S.%f";
pub const LOG_SIZE_MIN: usize = 1024 * 1024 * 2;
pub const LOG_SIZE_MAX: usize = 1024 * 1024 * 10;

lazy_static! {
    static ref DEBUG: Mutex<bool> = Mutex::new(false);
}

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LogLevel {
    Off,
    #[default]
    Default,
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Critical,
}

impl From<LogLevel> for LevelFilter {
    fn from(level: LogLevel) -> LevelFilter {
        match level {
            LogLevel::Off => LevelFilter::Off,
            LogLevel::Trace => LevelFilter::Trace,
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Warn => LevelFilter::Warn,
            LogLevel::Error => LevelFilter::Error,
            LogLevel::Critical => LevelFilter::Error,
            _ => LevelFilter::Info,
        }
    }
}

pub fn set_debug(value: bool) {
    let mut debug = DEBUG.lock().unwrap();
    *debug = value;
}

#[cfg(debug_assertions)]
pub fn is_debug() -> bool {
    let debug = DEBUG.lock().unwrap();
    *debug
}

#[cfg(not(debug_assertions))]
pub fn is_debug() -> bool {
    false
}

pub fn num_cpus() -> usize {
    if is_debug() {
        1
    } else {
        num_cpus::get()
    }
}
