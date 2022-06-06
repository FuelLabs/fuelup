use anyhow::Result;
use clap::Parser;

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
    install::install_one("fuelup", None);

    Ok(())
}
