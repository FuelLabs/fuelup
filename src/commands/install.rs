use anyhow::{bail, Result};
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

pub fn install_one(name: &str, version: Option<String>) -> Result<DownloadCfg> {
    let fuelup_bin_dir = fuelup_bin_dir();
    if !fuelup_bin_dir.is_dir() {
        fs::create_dir_all(&fuelup_bin_dir).expect("Unable to create fuelup directory");
    }

    let download_cfg = DownloadCfg::new(name, version).expect("Could not create download config");

    info!("Fetching {} {}", &download_cfg.name, &download_cfg.version);

    if download_file_and_unpack(&download_cfg).is_err() {
        bail!("{} {}", &download_cfg.name, &download_cfg.version)
    };

    if unpack_extracted_bins(&fuelup_bin_dir).is_err() {
        bail!("{} {}", &download_cfg.name, &download_cfg.version)
    };

    Ok(download_cfg)
}

pub fn install_all() -> Result<()> {
    for component in ["forc", "fuel-core"].iter() {
        install_one(component, None)?;
    }

    Ok(())
}

pub fn exec(command: InstallCommand) -> Result<()> {
    let InstallCommand { names } = command;

    let mut errored_bins = String::new();
    let mut installed_bins = String::new();
    let mut download_msg = String::new();

    if names.is_empty() {
        for name in POSSIBLE_COMPONENTS.iter() {
            write!(download_msg, "{} ", name)?;
        }
        info!("Downloading: {}", download_msg);
        install_all()?
    } else {
        let mut waiting_to_download = HashSet::new();
        let mut to_download = Vec::new();

        for name in names.iter() {
            if !waiting_to_download.contains(&name) && POSSIBLE_COMPONENTS.contains(&name.as_str())
            {
                to_download.push(name);
                write!(download_msg, "{} ", name)?;
            }
        }

        info!("Downloading: {}", download_msg);
        for name in to_download.iter() {
            if !waiting_to_download.contains(name) && POSSIBLE_COMPONENTS.contains(&name.as_str()) {
                waiting_to_download.insert(name);
                match install_one(name, None) {
                    Ok(cfg) => writeln!(installed_bins, "- {} {}", cfg.name, cfg.version)?,
                    Err(e) => writeln!(errored_bins, "- {}", e)?,
                };
            }
        }
    }

    if errored_bins.is_empty() {
        info!("\nInstalled:\n{}", installed_bins);
        info!("nThe Fuel toolchain is installed and up to date");
    } else if installed_bins.is_empty() {
        error!(
            "\nfuelup failed to install:\n{}\nYou might need to run `fuelup install` again.",
            errored_bins
        )
    } else {
        info!(
            "\nThe Fuel toolchain is partially installed.\nfuelup failed to install: {}\n\nYou might need to run `fuelup install` again.",
            errored_bins
        );
    };

    Ok(())
}
