use log4rs::{
    append::{
        console::ConsoleAppender,
        rolling_file::{
            policy::compound::{
                roll::fixed_window::FixedWindowRoller, trigger::size::SizeTrigger, CompoundPolicy,
            },
            RollingFileAppender,
        },
    },
    config::{Appender, Config, Logger, Root},
    encode::pattern::PatternEncoder,
    Handle,
};
use std::path::{PathBuf, MAIN_SEPARATOR};

use super::{LogLevel, LOG_DATE_FORMAT, LOG_SIZE_MAX, LOG_SIZE_MIN};
use crate::string::StringEx;

pub fn init(fle_name: &str) -> Handle {
    init_with(fle_name, LogLevel::Info, None)
}

pub fn init_with(file_name: &str, level: LogLevel, limit: Option<usize>) -> Handle {
    if file_name.is_empty() {
        panic!("File name is empty");
    }

    let path = PathBuf::from(file_name);
    let folder = match path.parent() {
        Some(folder) => folder.to_str().unwrap().suffix(MAIN_SEPARATOR),
        None => "".to_string(),
    };
    let base_name = path.file_stem().unwrap().to_str().unwrap().to_string();
    let roller_pattern = format!("{}{}.{{}}.old", folder, base_name);
    let console = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(&format!(
            "{{d({})}} | {{l:5.5}} | {{M}} | {{m}}{{n}}",
            LOG_DATE_FORMAT
        ))))
        .build();
    let size_trigger = SizeTrigger::new(
        limit
            .unwrap_or(LOG_SIZE_MAX)
            .clamp(LOG_SIZE_MIN, LOG_SIZE_MAX) as u64,
    );
    let fix_window_roller = FixedWindowRoller::builder()
        .build(&roller_pattern, 6)
        .unwrap();
    let policy = CompoundPolicy::new(Box::new(size_trigger), Box::new(fix_window_roller));
    let file = RollingFileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(&format!(
            "{{d({})}} | {{l:5.5}} | {{M}} | {{m}}{{D( | {{f}}:{{L}})}}{{n}}",
            LOG_DATE_FORMAT
        ))))
        .build(file_name, Box::new(policy))
        .unwrap();
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
                .additive(true)
                .build("file", level.into()),
        )
        .build(
            Root::builder()
                .appender("console")
                .appender("file")
                .build(level.into()),
        )
        .unwrap();
    log4rs::init_config(config).unwrap()
}
