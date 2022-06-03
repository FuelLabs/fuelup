use std::{collections::HashSet, fs};

use anyhow::Result;
use clap::Parser;
use tracing::info;

use crate::download::{
    download_file_and_unpack, fuelup_bin_dir, unpack_extracted_bins, DownloadCfg,
};

#[derive(Debug, Parser)]
pub struct InstallCommand {
    names: Vec<String>,
}

pub const POSSIBLE_COMPONENTS: [&str; 3] = ["forc", "fuel-core", "fuelup"]

pub fn install_one(name: &str, version: Option<String>) -> Result<()> {
    info!("\nDownloading the Fuel toolchain\n");

    let fuelup_bin_dir = fuelup_bin_dir();
    if !fuelup_bin_dir.is_dir() {
        fs::create_dir_all(&fuelup_bin_dir)?;
    }

    let download_cfg = DownloadCfg::new(name, version)?;

    info!("Fetching {} {}", &download_cfg.name, &download_cfg.version);
    download_file_and_unpack(&download_cfg)?;

    unpack_extracted_bins(&fuelup_bin_dir)?;

    Ok(())
}

pub fn install_all() -> Result<()> {
    for component in ["forc", "fuel-core"].iter() {
        install_one(component, None)?;
    }
    Ok(())
}

pub fn exec(command: InstallCommand) -> Result<()> {
    let InstallCommand { names } = command;

    if names.is_empty() {
        install_all()?
    } else {
        let mut waiting_to_download = HashSet::new();
        for name in names.iter() {
            if !waiting_to_download.contains(name) && POSSIBLE_COMPONENTS.contains(&name) {
                waiting_to_download.insert(name);
                install_one(&name, None)?;
            }
        }
    };

    Ok(())
}
