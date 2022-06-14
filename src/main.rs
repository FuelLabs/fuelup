use anyhow::Result;
use fuelup::download::component;
use fuelup::{fuelup_cli, proxy_cli};
use std::panic;
use std::path::PathBuf;

fn is_supported_component(component: &str) -> bool {
    ["forc", "fuel-core", "forc-fmt", "forc-lsp", "forc-explore"].contains(&component)
}

fn run() -> Result<()> {
    let arg0 = std::env::args().next().map(PathBuf::from);

    let process_name = arg0
        .as_ref()
        .and_then(|a| a.file_stem())
        .and_then(std::ffi::OsStr::to_str)
        .map(String::from);

    match process_name.as_deref() {
        Some(component::FUELUP) => fuelup_cli::fuelup_cli()?,
        Some(n) => {
            if is_supported_component(n) {
                proxy_cli::proxy_run(n)?;
            }
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
