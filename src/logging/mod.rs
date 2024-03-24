#[cfg(feature = "logging-slog")]
pub mod slog;

const LOG_DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S.%f";
const LOG_SIZE_MIN: usize = 1024 * 1024 * 2;
const LOG_SIZE_MAX: usize = 1024 * 1024 * 10;

#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    Defualt,
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}
