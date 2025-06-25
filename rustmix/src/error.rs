use thiserror::Error;

#[derive(Error, Debug)]
pub enum RmxError {
    #[error("Operation is canceled")]
    Canceled,

    #[error("Operation is not supported")]
    NotSupported,

    #[error("Not implemented")]
    NotImplemented,

    #[error("Invalid operation. {0}")]
    InvalidOperation(String),

    #[error("Operation timed out")]
    Timeout,

    #[error("No input was provided.")]
    NoInput,

    #[error("Argument error. {0}")]
    Argument(String),

    #[error("Invalid error. {0}")]
    Invalid(String),

    #[error("Missing error. {0}")]
    Missing(String),

    #[error("Http error. {0}")]
    Http(String),

    #[error("Network error. {0}")]
    Network(String),

    #[error("Command error {0}. {1}")]
    Command(i32, String),

    #[error("Item not found. {0}")]
    NotFound(String),

    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("Error exceeded. {0}")]
    Exceeded(String),

    #[error("Application exited with error {0}")]
    ExitCode(i32),
}
