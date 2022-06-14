use anyhow::{bail, Result};
use clap::Parser;

use crate::{
    download::{component, download_file_and_unpack, unpack_bins, DownloadCfg},
    path::fuelup_bin_dir,
};

#[derive(Debug, Parser)]
pub enum FuelupCommand {
    /// Updates fuelup
    Update,
}

#[derive(Debug, Parser)]
struct UpdateCommand {}

pub const FUELUP_VERSION: &str = concat!("v", clap::crate_version!());

pub fn self_update() -> Result<()> {
    let download_cfg = DownloadCfg::new(component::FUELUP, None)?;
    let fuelup_bin_dir = fuelup_bin_dir();
    if download_file_and_unpack(&download_cfg, &fuelup_bin_dir).is_err() {
        bail!("{} {}", &download_cfg.name, &download_cfg.version)
    };

    if unpack_bins(&fuelup_bin_dir, &fuelup_bin_dir).is_err() {
        bail!("{} {}", &download_cfg.name, &download_cfg.version)
    };

    Ok(())
}
