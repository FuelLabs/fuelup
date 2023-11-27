use anyhow::Result;
use fuelup::{
    fuelup_cli,
    logging::{init_tracing, log_command, log_environment},
    proxy_cli,
};
use std::{env, panic, path::PathBuf};
use tracing::error;

fn run() -> Result<()> {
    log_command();
    log_environment();
    let arg0 = env::args().next().map(PathBuf::from);

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

fn main() {
    let _guard = init_tracing();
    if run().is_err() {
        std::process::exit(1);
    }
}
