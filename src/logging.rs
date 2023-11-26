use crate::path::{fuelup_log_dir, FUELUP_DIR, FUELUP_HOME};
use std::env;
use tracing::{debug, level_filters::LevelFilter};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

pub fn log_command() {
    debug!("Command: {}", env::args().collect::<Vec<String>>().join(" "));
}

pub fn log_environment() {
    if let Some(val) = env::var_os("PATH") {
        if let Some(fuelup_path) =
            env::split_paths(&val).find(|p| p.to_string_lossy().contains(FUELUP_DIR))
        {
            debug!("PATH includes {}", fuelup_path.to_string_lossy());
        } else {
            debug!("PATH does not include {}", FUELUP_DIR);
        }
    }
    if let Some(val) = env::var_os("FUELUP_HOME") {
        debug!("FUELUP_HOME: {}", val.to_string_lossy());
    } else {
        debug!("FUELUP_HOME is not set");
    }
}

pub fn init_tracing() -> WorkerGuard {
    let file_appender = tracing_appender::rolling::hourly(fuelup_log_dir(), "fuelup.log");
    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::Registry::default()
        .with(
            tracing_subscriber::fmt::Layer::default()
                .with_writer(file_writer)
                .log_internal_errors(false)
                .with_target(false),
        )
        .with(
            tracing_subscriber::fmt::Layer::default()
                .with_writer(std::io::stdout)
                .without_time()
                .with_level(false)
                .with_target(false)
                .with_filter(LevelFilter::INFO),
        )
        .init();

    guard
}
