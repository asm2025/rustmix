use chrono::Local;
use slog::{o, Drain, Logger};
use slog_async::Async;
use slog_json::Json;
use slog_scope::GlobalLoggerGuard;
use slog_term::{CompactFormat, TermDecorator};

use super::super::io::file;
use super::LogLevel;

impl From<LogLevel> for slog::Level {
    fn from(level: LogLevel) -> slog::Level {
        match level {
            LogLevel::Trace => slog::Level::Trace,
            LogLevel::Debug => slog::Level::Debug,
            LogLevel::Warn => slog::Level::Warning,
            LogLevel::Error => slog::Level::Error,
            _ => slog::Level::Info,
        }
    }
}

pub fn init(fle_name: &str) -> GlobalLoggerGuard {
    init_with(fle_name, LogLevel::Info)
}

pub fn init_with(fle_name: &str, level: LogLevel) -> GlobalLoggerGuard {
    let decorator = TermDecorator::new().build();
    let term_drain = CompactFormat::new(decorator).build().fuse();
    let file_drain = Json::new(file::create(fle_name).unwrap())
        .add_key_value(o!("timestamp" => Local::now().to_rfc3339()))
        .build()
        .fuse();
    let drain = slog::Duplicate::new(term_drain, file_drain).fuse();
    let drain = Async::new(drain).build().fuse();
    let logger = Logger::root(drain.filter_level(level.into()).ignore_res(), o!());
    slog_scope::set_global_logger(logger)
}
