use log::LevelFilter;

#[cfg(feature = "log-log4rs")]
pub mod log4rs;
#[cfg(feature = "log-slog")]
pub mod slog;

pub const LOG_DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S.%f";
pub const LOG_SIZE_MIN: usize = 1024 * 1024 * 2;
pub const LOG_SIZE_MAX: usize = 1024 * 1024 * 10;

#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    Off,
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
