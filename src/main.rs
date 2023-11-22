use anyhow::Result;
use fuelup::path::fuelup_log_dir;
use fuelup::{fuelup_cli, proxy_cli};
use std::panic;
use std::path::PathBuf;
use tracing::error;
use tracing::level_filters::LevelFilter;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;

fn run() -> Result<()> {
    let arg0 = std::env::args().next().map(PathBuf::from);

    let process_name = arg0
        .as_ref()
        .and_then(|a| a.file_stem())
        .and_then(std::ffi::OsStr::to_str)
        .map(String::from);

    match process_name.as_deref() {
        Some(component::FUELUP) => {
            if let Err(e) = fuelup_cli::fuelup_cli() {
                error!("{}", e);
            }
        }
        Some(n) => {
            if let Err(e) = proxy_cli::proxy_run(n) {
                error!("{}", e);
            }
        }
        None => panic!("fuelup does not understand this command"),
    }
    Ok(())
}

fn init_tracing() -> WorkerGuard {
    let file_appender = tracing_appender::rolling::hourly(fuelup_log_dir(), "fuelup.log");
    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::Registry::default()
        .with(
            tracing_subscriber::fmt::Layer::default()
                .with_writer(file_writer)
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

fn main() {
    let _guard = init_tracing();
    if run().is_err() {
        std::process::exit(1);
    }
}
