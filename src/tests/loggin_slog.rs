use anyhow::Result;
use log::{debug, error, info, trace, warn};

use rustmix::{io::path::AsPath, logging::slog};

pub fn test_loggin_slog() -> Result<()> {
    println!("\nTesting loggin_slog functions...");

    let path = ("_logs", "test.log").as_path();
    let _gaurd = slog::init(&path);

    println!("Logging messages...");

    trace!("This is a trace message");
    debug!("This is a debug message");
    info!("This is an info message");
    warn!("This is a warning message");
    error!("This is an error message");

    println!("Check the log file at: {}", &path);

    Ok(())
}
