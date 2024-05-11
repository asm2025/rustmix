pub use log4rs::*;
use std::path::Path;

use self::{
    append::{
        console::ConsoleAppender,
        rolling_file::{
            policy::compound::{
                roll::fixed_window::FixedWindowRoller, trigger::size::SizeTrigger, CompoundPolicy,
            },
            RollingFileAppender,
        },
    },
    config::{runtime::ConfigBuilder, Appender, Root},
    encode::pattern::PatternEncoder,
};
use super::{LogLevel, LOG_DATE_FORMAT, LOG_SIZE_MAX, LOG_SIZE_MIN};
use crate::Result;

pub fn configure<T: AsRef<Path>>(
    file_name: T,
    level: LogLevel,
    limit: Option<usize>,
) -> Result<ConfigBuilder> {
    let file_name = file_name.as_ref();
    let folder = match file_name.parent() {
        Some(folder) => folder,
        None => Path::new(""),
    };
    let base_name = file_name.file_stem().unwrap().to_str().unwrap().to_string();
    let extension = file_name.extension().unwrap().to_str().unwrap().to_string();
    let roller_pattern = folder
        .join(format!("{}.{{}}.old.{}", base_name, extension))
        .to_string_lossy()
        .into_owned();
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
            config::Logger::builder()
                .appender("console")
                .build("console", level.into()),
        )
        .logger(
            config::Logger::builder()
                .appender("file")
                .build("file", level.into()),
        );
    Ok(config)
}

pub fn from_config(config: Config) -> Result<Handle> {
    log4rs::init_config(config).map_err(Into::into)
}

pub fn build<T: AsRef<Path>>(file_name: T) -> Result<Handle> {
    build_with(file_name, LogLevel::Info, None)
}

pub fn build_with<T: AsRef<Path>>(
    file_name: T,
    level: LogLevel,
    limit: Option<usize>,
) -> Result<Handle> {
    let config = configure(file_name, level, limit)?.build(
        Root::builder()
            .appender("console")
            .appender("file")
            .build(level.into()),
    )?;
    log4rs::init_config(config).map_err(Into::into)
}

pub fn from_file<T: AsRef<Path>>(yaml_file_name: T) -> Result<()> {
    log4rs::init_file(yaml_file_name, Default::default()).map_err(Into::into)
}
