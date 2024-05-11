use log::{debug, error, info, trace, warn};

use rustmix::{
    io::{directory, path::AsPath},
    log4rs, Result,
};

pub fn test_log4rs(from_config_file: bool) -> Result<()> {
    println!("\nTesting log4rs functions...");

    if from_config_file {
        let path = directory::current().join("files/log/log4rs.yaml");
        println!("Building loggers from file: {}", &path.display());
        println!("file exist? {}", &path.exists());
        log4rs::from_file(&path)?;
        println!("Logger was built");
        log_a_few_messages("Messages configured logger from a yaml file:");
        println!("Check the log file: {}", &path.display());
    } else {
        let path = ("_logs", "test.log").as_path();
        println!("Building loggers from code: {}", &path);
        log4rs::build(&path)?;
        println!("Logger was built");
        log_a_few_messages("Messages configured logger programmatically:");
        println!("Check the log file: {}", &path);
    }

    Ok(())
}

fn log_a_few_messages(header: &str) {
    println!("Logging messages...");
    error!("{}", header);
    trace!("This is a trace message");
    debug!("This is a debug message");
    info!("This is an info message");
    warn!("This is a warning message");
    error!("This is an error message");
    error!("Filtered out level will not show up in the log file");
    error!("------------------------------------------------------------------");
}
