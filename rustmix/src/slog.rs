use chrono::Local;
pub use file_rotate::{compression::Compression, suffix::AppendCount, ContentLimit, FileRotate};
pub use slog::{self as lib};
pub use slog_async::*;
use slog_json::Json;
pub use slog_scope::GlobalLoggerGuard;
pub use slog_term::{Decorator, PlainSyncDecorator};
use std::io;

use super::{LogLevel, LOG_DATE_FORMAT, LOG_SIZE_MAX, LOG_SIZE_MIN};
use crate::{error::ArgumentIsNullOrEmptyError, Result};

impl From<LogLevel> for slog::Level {
    fn from(level: LogLevel) -> slog::Level {
        match level {
            LogLevel::Off => slog::Level::Critical,
            LogLevel::Trace => slog::Level::Trace,
            LogLevel::Debug => slog::Level::Debug,
            LogLevel::Warn => slog::Level::Warning,
            LogLevel::Error => slog::Level::Error,
            LogLevel::Critical => slog::Level::Critical,
            _ => slog::Level::Info,
        }
    }
}

pub struct CustomDecorator<D: Decorator> {
    decorator: D,
}

impl<D: Decorator> CustomDecorator<D> {
    fn new(decorator: D) -> Self {
        Self { decorator }
    }
}

impl<D: Decorator> lib::Drain for CustomDecorator<D> {
    type Ok = ();
    type Err = io::Error;

    fn log(&self, record: &lib::Record, values: &lib::OwnedKVList) -> io::Result<()> {
        self.decorator.with_record(record, values, |d| {
            writeln!(d, "{}| {} | {}", record.level(), record.tag(), record.msg())
        })
    }
}

pub fn build(file_name: &str) -> Result<GlobalLoggerGuard> {
    build_with(file_name, LogLevel::Info, None)
}

pub fn build_with(
    file_name: &str,
    level: LogLevel,
    limit: Option<usize>,
) -> Result<GlobalLoggerGuard> {
    if file_name.is_empty() {
        return Err(ArgumentIsNullOrEmptyError("file_name".to_string()).into());
    }

    let decorator = PlainSyncDecorator::new(io::stdout());
    let drain = CustomDecorator::new(decorator);
    let logger = FileRotate::new(
        file_name,
        AppendCount::new(6),
        ContentLimit::Bytes(
            limit
                .unwrap_or(LOG_SIZE_MAX)
                .clamp(LOG_SIZE_MIN, LOG_SIZE_MAX),
        ),
        Compression::None,
        #[cfg(unix)]
        None,
    );
    let file_drain = lib::Drain::fuse(
        Json::new(logger)
            .add_key_value(lib::o!("timestamp" => lib::FnValue(|_| {
                Local::now().format(LOG_DATE_FORMAT).to_string()
            })))
            .add_key_value(lib::o!("level" => lib::FnValue(|r| {
                r.level().as_str().to_string()
            })))
            .add_key_value(lib::o!("tag" => lib::FnValue(|r| {
                r.tag().to_string()
            })))
            .add_key_value(lib::o!("message" => lib::FnValue(|r| {
                r.msg().to_string()
            })))
            .add_key_value(lib::o!("arguments" => lib::FnValue(|_| {
                None::<&str>
            })))
            .add_key_value(lib::o!("location" => lib::FnValue(|r| {
                if cfg!(debug_assertions) {
                    Some(format!("{}:{}", &r.file(), &r.line()))
                } else {
                    None
                }
            })))
            .build(),
    );
    let drain = lib::Drain::fuse(slog::Duplicate::new(drain, file_drain));
    let drain = lib::Drain::fuse(
        Async::new(lib::Drain::ignore_res(lib::Drain::filter_level(
            drain,
            level.into(),
        )))
        .build(),
    );
    let logger = lib::Logger::root(drain, lib::o!());
    let guard = slog_scope::set_global_logger(logger);
    slog_stdlog::init()?;
    Ok(guard)
}
