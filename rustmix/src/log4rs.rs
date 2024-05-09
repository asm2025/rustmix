use lib::{
    append::{
        console::ConsoleAppender,
        rolling_file::{
            policy::compound::{
                roll::fixed_window::FixedWindowRoller, trigger::size::SizeTrigger, CompoundPolicy,
            },
            RollingFileAppender,
        },
    },
    config::{runtime::ConfigBuilder, Appender, Config, Logger, Root},
    encode::pattern::PatternEncoder,
    Handle,
};
pub use log4rs::{self as lib};
use std::path::{PathBuf, MAIN_SEPARATOR};

use super::{LogLevel, LOG_DATE_FORMAT, LOG_SIZE_MAX, LOG_SIZE_MIN};
use crate::{error::ArgumentIsNullOrEmptyError, string::StringEx, Result};

pub fn configure(file_name: &str, level: LogLevel, limit: Option<usize>) -> Result<ConfigBuilder> {
    if file_name.is_empty() {
        return Err(ArgumentIsNullOrEmptyError("file_name").into());
    }

    let path = PathBuf::from(file_name);
    let folder = match path.parent() {
        Some(folder) => folder.to_str().unwrap().suffix(MAIN_SEPARATOR),
        None => "".to_string(),
    };
    let base_name = path.file_stem().unwrap().to_str().unwrap().to_string();
    let extension = path.extension().unwrap().to_str().unwrap().to_string();
    let roller_pattern = format!("{}{}.{{}}.old.{}", folder, base_name, extension);
    let console = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{l:5.5}| {M} | {m}{n}")))
        .build();
    let size_trigger = SizeTrigger::new(
        limit
            .unwrap_or(LOG_SIZE_MAX)
            .clamp(LOG_SIZE_MIN, LOG_SIZE_MAX) as u64,
    );
    let fix_window_roller = FixedWindowRoller::builder().build(&roller_pattern, 6)?;
    let policy = CompoundPolicy::new(Box::new(size_trigger), Box::new(fix_window_roller));
    let file = RollingFileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(&format!(
            "{{d({})}} | {{l:5.5}} | {{M}} | {{m}}{{D( | {{f}}:{{L}})}}{{n}}",
            LOG_DATE_FORMAT
        ))))
        .append(true)
        .build(file_name, Box::new(policy))?;
    let config = Config::builder()
        .appender(Appender::builder().build("console", Box::new(console)))
        .appender(Appender::builder().build("file", Box::new(file)))
        .logger(
            Logger::builder()
                .appender("console")
                .build("console", level.into()),
        )
        .logger(
            Logger::builder()
                .appender("file")
                .build("file", level.into()),
        );
    Ok(config)
}

pub fn from_config(config: Config) -> Result<Handle> {
    log4rs::init_config(config).map_err(Into::into)
}

pub fn build(file_name: &str) -> Result<Handle> {
    build_with(file_name, LogLevel::Info, None)
}

pub fn build_with(file_name: &str, level: LogLevel, limit: Option<usize>) -> Result<Handle> {
    let config = configure(file_name, level, limit)?.build(
        Root::builder()
            .appender("console")
            .appender("file")
            .build(level.into()),
    )?;
    log4rs::init_config(config).map_err(Into::into)
}

pub fn from_file(yaml_file_name: &str) -> Result<()> {
    if yaml_file_name.is_empty() {
        return Err(ArgumentIsNullOrEmptyError("yaml_file_name").into());
    }

    log4rs::init_file(yaml_file_name, Default::default()).map_err(Into::into)
}
