use anyhow::{bail, Result};
use clap::Parser;
use std::fmt::Write;
use tracing::{error, info};

use crate::{
    commands::install::install_one,
    download::{component, DownloadCfg},
};

pub mod toolchain {
    pub const LATEST: &str = "latest";
}

#[derive(Debug, Parser)]
pub enum ToolchainCommand {
    /// Install or update a given toolchain
    ///
    /// Currently, we only support installation of the 'latest' toolchain:
    /// `fuelup toolchain install latest`
    Install(InstallCommand),
}

#[derive(Debug, Parser)]
pub struct InstallCommand {
    /// Toolchain name [possible values: latest]
    toolchain: String,
}

#[derive(Debug)]
pub enum ToolchainName {
    LATEST,
}

pub struct Toolchain {
    pub name: ToolchainName,
}

pub fn exec(command: ToolchainCommand) -> Result<()> {
    match command {
        ToolchainCommand::Install(command) => install(command)?,
    };

    Ok(())
}

pub fn install(command: InstallCommand) -> Result<()> {
    let InstallCommand { toolchain } = command;

    if ![toolchain::LATEST].contains(&toolchain.as_ref()) {
        bail!(
            "Invalid toolchain: {} [possible values: {}]",
            toolchain,
            toolchain::LATEST
        );
    }

    let mut errored_bins = String::new();
    let mut installed_bins = String::new();
    let mut download_msg = String::new();

    let mut cfgs: Vec<DownloadCfg> = Vec::new();

    for component in [component::FORC, component::FUEL_CORE, component::FUELUP].iter() {
        write!(download_msg, "{} ", component)?;
        let download_cfg: DownloadCfg = DownloadCfg::new(component, None)?;
        cfgs.push(download_cfg);
    }

    info!("Downloading: {}", download_msg);
    for cfg in cfgs {
        match install_one(cfg) {
            Ok(cfg) => writeln!(installed_bins, "- {} {}", cfg.name, cfg.version)?,
            Err(e) => writeln!(errored_bins, "- {}", e)?,
        };
    }

    if errored_bins.is_empty() {
        info!("\nInstalled:\n{}", installed_bins);
        info!("\nThe Fuel toolchain is installed and up to date");
    } else if installed_bins.is_empty() {
        error!("\nfuelup failed to install:\n{}", errored_bins)
    } else {
        info!(
            "\nThe Fuel toolchain is partially installed.\nfuelup failed to install: {}",
            errored_bins
        );
    };

    Ok(())
}
