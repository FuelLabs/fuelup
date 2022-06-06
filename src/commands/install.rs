use anyhow::{bail, Error, Result};
use clap::Parser;
use std::collections::HashSet;
use std::fmt::Write;
use std::fs;
use tracing::{error, info};

use crate::download::{
    download_file_and_unpack, fuelup_bin_dir, unpack_extracted_bins, DownloadCfg,
};

#[derive(Debug, Parser)]
pub struct InstallCommand {
    names: Vec<String>,
}

pub const POSSIBLE_COMPONENTS: [&str; 3] = ["forc", "fuel-core", "fuelup"];

pub fn install_one(name: &str, version: Option<String>) -> Option<DownloadCfg> {
    info!("\nDownloading the Fuel toolchain\n");

    let fuelup_bin_dir = fuelup_bin_dir();
    if !fuelup_bin_dir.is_dir() {
        fs::create_dir_all(&fuelup_bin_dir).expect("Unable to create fuelup directory");
    }

    let download_cfg = DownloadCfg::new(name, version).expect("Could not create download config");

    info!("Fetching {} {}", &download_cfg.name, &download_cfg.version);

    if let Err(_) = download_file_and_unpack(&download_cfg) {
        error!(
            "Failed to download {} {}",
            &download_cfg.name, &download_cfg.version
        );
        return None;
    };

    if let Err(_) = unpack_extracted_bins(&fuelup_bin_dir) {
        error!(
            "Failed to unpack {} {}",
            &download_cfg.name, &download_cfg.version
        );
    };

    Ok(download_cfg)
}

pub fn install_all() -> Result<()> {
    for component in ["forc", "fuel-core"].iter() {
        let downloaded = install_one(component, None);
    }

    Ok(())
}

pub fn exec(command: InstallCommand) -> Result<()> {
    let InstallCommand { names } = command;

    let errored_bins = String::new();
    let downloaded_bins = String::new();

    if names.is_empty() {
        install_all()?
    } else {
        let mut waiting_to_download = HashSet::new();
        for name in names.iter() {
            if !waiting_to_download.contains(name) && POSSIBLE_COMPONENTS.contains(&name) {
                waiting_to_download.insert(name);
                match install_one(&name, None) {
                    Ok(cfg) => write!(downloaded_bins, "{} {}", cfg.name, cfg.version),
                    Err(cfg) => write!(errored_bins, "{} {}", cfg.name, cfg.version),
                };
            }
        }
    }

    if errored_bins_message.is_empty() {
        info!("\nInstalled: {}", installed_bins_message);
        info!("\nThe Fuel toolchain is installed and up to date");
    } else if installed_bins_message.is_empty() {
        error!(
            "\nfuelup failed to install: {}\n\nYou might need to run `fuelup install` again.",
            errored_bins_message
        )
    } else {
        info!(
            "\nThe Fuel toolchain is partially installed.\nfuelup failed to install: {}\n\nYou might need to run `fuelup install` again.",
            errored_bins_message
        );
    };

    Ok(())
}
