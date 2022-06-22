use anyhow::{bail, Result};
use fuelup::component;
use fuelup::{fuelup_cli, proxy_cli};
use std::panic;
use std::path::PathBuf;
use tracing::error;

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
        Some(n) if component::SUPPORTED_COMPONENTS.contains(&n) => {
            if let Err(e) = proxy_cli::proxy_run(n) {
                error!("{}", e);
            }
        }
        Some(n) => {
            bail!(
                "fuelup invoked with unexpected command or component {:?}",
                n
            )
        }
        None => panic!("fuelup does not understand this command"),
    }
    Ok(())
}

fn main() {
    let format = tracing_subscriber::fmt::format()
        .without_time()
        .with_level(false)
        .with_target(false);

    tracing_subscriber::fmt().event_format(format).init();

    if run().is_err() {
        std::process::exit(1);
    }
}
