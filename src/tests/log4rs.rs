use anyhow::Result;
use log::{debug, error, info, trace, warn};

use rustmix::{io::path::AsPath, logging::log4rs};

pub fn test_log4rs() -> Result<()> {
    println!("\nTesting log4rs functions...");

    let path = ("_logs", "test.log").as_path();
    let _gaurd = log4rs::init(&path);

    println!("Logging messages...");

    trace!("This is a trace message");
    debug!("This is a debug message");
    info!("This is an info message");
    warn!("This is a warning message");
    error!("This is an error message");

    println!("Check the log file at: {}", &path);

    Ok(())
}
