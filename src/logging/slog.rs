use anyhow::Result;
use chrono::Local;
use file_rotate::{compression::Compression, suffix::AppendCount, ContentLimit, FileRotate};
use serde_json::json;
use slog::{o, Drain, Logger, OwnedKVList, Record, Serializer, KV};
use slog_async::Async;
use slog_scope::GlobalLoggerGuard;
use slog_term::{FullFormat, TermDecorator};
use std::{
    collections::HashMap,
    io::Write,
    sync::{Arc, Mutex},
};

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

struct MapSerializer<'a>(&'a mut HashMap<String, String>);

impl<'a> Serializer for MapSerializer<'a> {
    fn emit_arguments(
        &mut self,
        key: slog::Key,
        val: &std::fmt::Arguments,
    ) -> Result<(), slog::Error> {
        self.0.insert(key.into(), val.to_string());
        Ok(())
    }
}

struct TsvDrain {
    logger: Arc<Mutex<FileRotate<AppendCount>>>,
}

impl TsvDrain {
    fn new(logger: FileRotate<AppendCount>) -> Self {
        Self {
            logger: Arc::new(Mutex::new(logger)),
        }
    }
}

impl Drain for TsvDrain {
    type Ok = ();
    type Err = std::io::Error;

    fn log(&self, record: &Record, values: &OwnedKVList) -> Result<Self::Ok, Self::Err> {
        let mut map = HashMap::new();
        let mut serializer = MapSerializer(&mut map);
        values.serialize(record, &mut serializer)?;
        let json = if !map.is_empty() {
            format!("\n{}", json!(&map))
        } else {
            "".to_string()
        };
        let mut logger = self.logger.lock()?;
        writeln!(
            &mut logger,
            "{} | {:5.5} | {} | {}{}",
            Local::now().format(LOG_DATE_FORMAT),
            record.level(),
            record.tag(),
            record.msg(),
            json
        )?;
        Ok(())
    }
}

pub fn init(fle_name: &str) -> Result<GlobalLoggerGuard> {
    init_with(fle_name, LogLevel::Info, None)
}

pub fn init_with(
    fle_name: &str,
    level: LogLevel,
    limit: Option<usize>,
) -> Result<GlobalLoggerGuard> {
    if file_name.is_empty() {
        panic!("File name is empty");
    }

    let decorator = TermDecorator::new().build();
    let term_drain = FullFormat::new(decorator)
        .use_custom_timestamp(|out| {})
        .build()
        .fuse();
    let logger = FileRotate::new(
        fle_name,
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
    let file_drain = TsvDrain::new(logger).fuse();
    let drain = slog::Duplicate::new(term_drain, file_drain).fuse();
    let drain = Async::new(drain.filter_level(level.into()).ignore_res())
        .build()
        .fuse();
    let logger = Logger::root(drain, o!());
    slog_stdlog::init()?;
    slog_scope::set_global_logger(logger)
}
