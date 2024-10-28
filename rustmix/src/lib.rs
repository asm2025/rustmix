mod app;
#[cfg(feature = "audio")]
pub mod audio;
#[cfg(feature = "language")]
pub mod language;
#[cfg(feature = "log")]
pub mod log;
#[cfg(feature = "vision")]
pub mod vision;
pub use self::app::*;
pub mod date;
pub mod error;
pub mod input;
pub mod io;
pub mod random;
pub mod string;
pub mod threading;
pub mod vpn;
pub mod web;

pub use ::backoff::*;

use lazy_static::lazy_static;
use std::sync::RwLock;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

lazy_static! {
    static ref DEBUG: RwLock<bool> = RwLock::new(false);
}

pub fn set_debug(value: bool) {
    let mut debug = DEBUG.write().unwrap();
    *debug = value;
}

#[cfg(debug_assertions)]
pub fn is_debug() -> bool {
    *DEBUG.read().unwrap()
}

#[cfg(not(debug_assertions))]
pub fn is_debug() -> bool {
    false
}

pub trait CallbackHandler<T> {
    fn starting(&self);
    fn update(&self, data: T);
    fn completed(&self);
}

pub mod ai {
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum SourceSize {
        Tiny,
        #[default]
        Small,
        Base,
        Medium,
        Large,
    }
}

pub mod system {
    pub fn num_cpus() -> usize {
        if crate::is_debug() {
            1
        } else {
            num_cpus::get()
        }
    }
}
