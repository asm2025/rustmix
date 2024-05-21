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
#[error("Operation is canceled")]
pub struct CanceledError;

#[derive(Error, Debug)]
#[error("Operation is not supported")]
pub struct NotSupportedError;

#[derive(Error, Debug)]
#[error("Invalid operation. {0}")]
pub struct InvalidOperationError(pub String);

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
#[error("Queue already dropped")]
pub struct QueueDroppedError;

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
#[error("Argument '{0}' is null or empty")]
pub struct ArgumentIsNullOrEmptyError(pub String);

#[derive(Error, Debug)]
#[error("No content")]
pub struct NoContentError;

#[derive(Error, Debug)]
#[error("HtmlElement '{0}' not found")]
pub struct ElementNotFoundError(pub String);

#[derive(Error, Debug)]
#[error("Invalid HTTP response")]
pub struct InvalidResponseError;

#[derive(Error, Debug)]
#[error("Not implemented")]
pub struct NotImplementedError;

#[derive(Error, Debug)]
#[error("Error {0}. {1}")]
pub struct CommandError(pub i32, pub String);

#[derive(Error, Debug)]
#[error("Invalid command response")]
pub struct InvalidCommandResponseError;

#[derive(Error, Debug)]
#[error("VPN error. {0}")]
pub struct VPNError(pub String);

#[derive(Error, Debug)]
#[error("Unknown VPN response {0}")]
pub struct UnknownVPNResponseError(pub String);
