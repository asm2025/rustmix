#[cfg(feature = "logging-slog")]
pub mod slog;

pub enum LogLevel {
    Defualt,
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}
