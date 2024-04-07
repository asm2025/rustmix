use std::{error::Error, fmt};

#[derive(Debug)]
pub struct CancelledError;

impl fmt::Display for CancelledError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Operation is cancelled")
    }
}

impl Error for CancelledError {}

#[derive(Debug)]
pub struct TimedoutError;

impl fmt::Display for TimedoutError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Operation timed out")
    }
}

impl Error for TimedoutError {}

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;
