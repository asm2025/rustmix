use anyhow::Result;
use chrono::Local;
use file_rotate::{compression::Compression, suffix::AppendCount, ContentLimit, FileRotate};
use slog::{o, Drain, FnValue, Logger};
use slog_async::Async;
use slog_json::Json;
use slog_scope::GlobalLoggerGuard;
use slog_term::{FullFormat, TermDecorator};

use super::{LogLevel, LOG_DATE_FORMAT, LOG_SIZE_MAX, LOG_SIZE_MIN};

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

pub fn init(file_name: &str) -> Result<GlobalLoggerGuard> {
    init_with(file_name, LogLevel::Info, None)
}

pub fn init_with(
    file_name: &str,
    level: LogLevel,
    limit: Option<usize>,
) -> Result<GlobalLoggerGuard> {
    if file_name.is_empty() {
        panic!("File name is empty");
    }

    let decorator = TermDecorator::new().force_color().build();
    let drain = FullFormat::new(decorator)
        .use_custom_timestamp(|_| Ok(()))
        .build()
        .fuse();
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
    let file_drain = Json::new(logger)
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
        .build()
        .fuse();
    let drain = slog::Duplicate::new(drain, file_drain).fuse();
    let drain = Async::new(drain.filter_level(level.into()).ignore_res())
        .build()
        .fuse();
    let logger = Logger::root(drain, o!());
    let guard = slog_scope::set_global_logger(logger);
    slog_stdlog::init()?;
    Ok(guard)
}
