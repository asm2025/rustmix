use std::{error::Error, fmt};

use crate::is_debug;

pub trait ExceptionStr {
    fn get_string(&self) -> String;
}

impl<E: Error + ?Sized> ExceptionStr for E {
    fn get_string(&self) -> String {
        if is_debug() {
            return format!("{:?}", self);
        }

        self.to_string()
    }
}

pub trait ExceptionEx {
    fn get_message(&self) -> String;
}

impl<E: Error + ExceptionStr> ExceptionEx for E {
    fn get_message(&self) -> String {
        if !is_debug() {
            return self.get_string();
        }

        let mut msg = String::new();
        let mut e: Option<&dyn Error> = Some(self);

        while let Some(err) = e {
            if msg.len() > 0 {
                msg.push_str("\n");
            }

            msg.push_str(&err.get_string());
            e = err.source();
        }

        msg
    }
}

pub trait AnyhowEx {
    fn get_message(&self) -> String;
}

impl AnyhowEx for anyhow::Error {
    fn get_message(&self) -> String {
        if let Some(err) = self.source() {
            return err.get_message();
        }

        return self.get_string();
    }
}

#[derive(Debug)]
pub struct CancelledError;

impl fmt::Display for CancelledError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Operation is cancelled")
    }
}

impl Error for CancelledError {}

#[derive(Debug)]
pub struct QueueCompletedError;

impl fmt::Display for QueueCompletedError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Queue is already completed")
    }
}

impl Error for QueueCompletedError {}

#[derive(Debug)]
pub struct TimedoutError;

impl fmt::Display for TimedoutError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Operation timed out")
    }
}

impl Error for TimedoutError {}

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;
