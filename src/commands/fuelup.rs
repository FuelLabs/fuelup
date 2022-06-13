use anyhow::Result;
use clap::Parser;

use crate::{
    download::{component, DownloadCfg},
    toolchain::Toolchain,
};

use super::toolchain::toolchain;

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
    let toolchain = Toolchain::new(toolchain::LATEST, None)?;
    toolchain.add_component(download_cfg)?;

    Ok(())
}
