use std::error::Error as StdError;
use thiserror::Error;

use crate::is_debug;

pub trait ErrorStr {
    fn get_string(&self) -> String;
}

impl<E: StdError + ?Sized> ErrorStr for E {
    fn get_string(&self) -> String {
        if is_debug() {
            return format!("{:?}", self);
        }

        self.to_string()
    }
}

pub trait ErrorEx {
    fn get_message(&self) -> String;
}

impl<E: StdError + ?Sized> ErrorEx for E {
    fn get_message(&self) -> String {
        if !is_debug() {
            return self.get_string();
        }

        let mut msg = String::new();
        let mut e: Option<&dyn StdError> = Some(&self);

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

#[derive(Error, Debug)]
#[error("Operation is cancelled")]
pub struct CancelledError;

#[derive(Error, Debug)]
#[error("Operation is not supported")]
pub struct NotSupportedError;

#[derive(Error, Debug)]
#[error("Invalid operation. {0}")]
pub struct InvalidOperationError(pub &'static str);

#[derive(Error, Debug)]
#[error("Operation timed out")]
pub struct TimedoutError;

#[derive(Error, Debug)]
#[error("Queue already started")]
pub struct QueueStartedError;

#[derive(Error, Debug)]
#[error("Queue already completed")]
pub struct QueueCompletedError;

#[derive(Error, Debug)]
#[error("No input provided")]
pub struct NoInputError;

#[derive(Error, Debug)]
#[error("Invalid input")]
pub struct InvalidInputError;

#[derive(Error, Debug)]
#[error("Not confirmed")]
pub struct NotConfirmError;

#[derive(Error, Debug)]
#[error("Argument is null or empty: {0}")]
pub struct ArgumentIsNullOrEmptyError(pub &'static str);

#[derive(Error, Debug)]
#[error("No content")]
pub struct NoContentError;

#[derive(Error, Debug)]
#[error("HtmlElement not found: {0}")]
pub struct ElementNotFoundError(pub &'static str);

#[derive(Error, Debug)]
#[error("Invalid HTTP response")]
pub struct InvalidResponseError;

#[derive(Error, Debug)]
#[error("Not implemented")]
pub struct NotImplementedError;
