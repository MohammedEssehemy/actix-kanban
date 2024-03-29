use log::{debug, error, info, trace, warn};
use std::env;
use std::fs;

pub fn init() -> Result<(), fern::InitError> {
    // pull log level from env
    let log_level = env::var("LOG_LEVEL").unwrap_or("INFO".into());
    let log_level = log_level
        .parse::<log::LevelFilter>()
        .unwrap_or(log::LevelFilter::Info);

    let mut builder = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{:?}][{}][{}] {}",
                chrono::Utc::now(),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log_level)
        .chain(std::io::stdout());

    // also log to file if one is provided via env
    if let Ok(log_file) = env::var("LOG_FILE") {
        let log_file = fs::File::create(log_file)?;
        builder = builder.chain(log_file);
    }

    // globally apply logger
    builder.apply()?;

    trace!("TRACE output enabled");
    debug!("DEBUG output enabled");
    info!("INFO output enabled");
    warn!("WARN output enabled");
    error!("ERROR output enabled");

    Ok(())
}
