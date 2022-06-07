use anyhow::Result;
use clap::Parser;

use crate::download::DownloadCfg;

use super::install;

#[derive(Debug, Parser)]
pub enum FuelupCommand {
    /// Updates fuelup
    Update,
}

#[derive(Debug, Parser)]
struct UpdateCommand {}

pub const FUELUP_VERSION: &str = concat!("v", clap::crate_version!());

pub fn self_update() -> Result<()> {
    let download_cfg = DownloadCfg::new("fuelup", None)?;
    install::install_one(download_cfg)?;

    Ok(())
}
