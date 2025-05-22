use chrono::Local;
pub use file_rotate::{compression::Compression, suffix::AppendCount, ContentLimit, FileRotate};
pub use slog::*;
pub use slog_async::*;
use slog_json::Json;
pub use slog_scope::GlobalLoggerGuard;
pub use slog_term::{Decorator, PlainSyncDecorator};
use std::{io, path::Path};

use super::{LogLevel, LOG_DATE_FORMAT, LOG_SIZE_MAX, LOG_SIZE_MIN};
use crate::Result as CommonResult;

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

impl<D: Decorator> Drain for CustomDecorator<D> {
    type Ok = ();
    type Err = io::Error;

    fn log(&self, record: &Record, values: &OwnedKVList) -> io::Result<()> {
        self.decorator.with_record(record, values, |d| {
            writeln!(d, "{}| {} | {}", record.level(), record.tag(), record.msg())
        })
    }
}

pub fn build<T: AsRef<Path>>(file_name: T) -> CommonResult<GlobalLoggerGuard> {
    build_with(file_name, LogLevel::Info, None)
}

pub fn build_with<T: AsRef<Path>>(
    file_name: T,
    level: LogLevel,
    limit: Option<usize>,
) -> CommonResult<GlobalLoggerGuard> {
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
    let file_drain = Drain::fuse(
        Json::new(logger)
            .add_key_value(o!("timestamp" => FnValue(|_| {
                Local::now().format(LOG_DATE_FORMAT).to_string()
            })))
            .add_key_value(o!("level" => FnValue(|r| {
                r.level().as_str().to_string()
            })))
            .add_key_value(o!("tag" => FnValue(|r| {
                r.tag().to_string()
            })))
            .add_key_value(o!("message" => FnValue(|r| {
                r.msg().to_string()
            })))
            .add_key_value(o!("arguments" => FnValue(|_| {
                None::<&str>
            })))
            .add_key_value(o!("location" => FnValue(|r| {
                if cfg!(debug_assertions) {
                    Some(format!("{}:{}", &r.file(), &r.line()))
                } else {
                    None
                }
            })))
            .build(),
    );
    let drain = Drain::fuse(slog::Duplicate::new(drain, file_drain));
    let drain = Drain::fuse(
        Async::new(Drain::ignore_res(Drain::filter_level(drain, level.into()))).build(),
    );
    let logger = Logger::root(drain, o!());
    let guard = slog_scope::set_global_logger(logger);
    slog_stdlog::init()?;
    Ok(guard)
}
