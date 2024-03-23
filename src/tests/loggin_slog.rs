use anyhow::Result;
use log::{debug, error, info, trace, warn};

use rustmix::{
    io::path::AsPath,
    logging::{slog, LogLevel},
};

pub fn test_loggin_slog() -> Result<()> {
    println!("\nTesting loggin_slog functions...");
    let _gaurd = slog::init_with(&("_logs", "test.log").as_path(), LogLevel::Debug);

    trace!("This is a trace message");
    debug!("This is a debug message");
    info!("This is an info message");
    warn!("This is a warning message");
    error!("This is an error message");

    Ok(())
}
