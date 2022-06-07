use anyhow::Result;
use clap::Parser;
use tracing::info;

use crate::{constants::POSSIBLE_COMPONENTS, download::DownloadCfg};

#[derive(Debug, Parser)]
pub struct CheckCommand {}

pub const FUELUP_VERSION: &str = concat!("v", clap::crate_version!());

pub fn exec() -> Result<()> {
    for component in POSSIBLE_COMPONENTS.iter() {
        let output = std::process::Command::new(component)
            .arg("--version")
            .output()
            .unwrap_or_else(|_| panic!("Could not run {} --version", component))
            .stdout;

        let output = String::from_utf8_lossy(&output);
        let version = "v".to_owned() + output.split_whitespace().collect::<Vec<&str>>()[1];

        let download_cfg: DownloadCfg = DownloadCfg::new(component, None)
            .unwrap_or_else(|_| panic!("Could not create download config for {}", component));

        if version == download_cfg.version {
            info!("{} - up to date: {}", component, version);
        } else {
            info!(
                "{} - update available: {} -> {}",
                component, version, download_cfg.version
            );
        }
    }

    Ok(())
}
