use anyhow::Result;
use log::{debug, error, info, trace, warn};

use ::{io::path::AsPath, logging::slog};

pub fn test_slog() -> Result<()> {
    println!("\nTesting slog functions...");

    println!("Building loggers from code...");
    let path = ("_logs", "test.log").as_path();
    let _gaurd = slog::init(&path)?;
    println!("Logger was built");
    println!("Logging messages...");
    error!("Messages configured logger programmatically:");
    trace!("This is a trace message");
    debug!("This is a debug message");
    info!("This is an info message");
    warn!("This is a warning message");
    error!("This is an error message");
    error!("Filtered out level will not show up in the log file");
    error!("------------------------------------------------------------------");
    println!("Check the log file at: {}", &path);

    Ok(())
}
