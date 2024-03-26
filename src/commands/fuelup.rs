use anyhow::{bail, Result};
use clap::Parser;

use crate::ops::fuelup_self::{self_uninstall, self_update};

#[derive(Debug, Parser)]
pub enum FuelupCommand {
    /// Updates fuelup
    Update(UpdateCommand),
    /// Uninstall fuelup
    Uninstall(UninstallCommand),
}

#[derive(Debug, Parser)]
pub struct UninstallCommand {
    #[clap(long, short)]
    pub force: bool,
}

#[derive(Debug, Parser)]
pub struct UpdateCommand {
    #[clap(long, short)]
    pub force: bool,
}

pub fn update_exec(force: bool) -> Result<()> {
    if let Err(e) = self_update(force) {
        bail!("fuelup failed to update itself: {}", e)
    };

    Ok(())
}

pub fn remove_exec(force: bool) -> Result<()> {
    if let Err(e) = self_uninstall(force) {
        bail!("fuelup failed to update itself: {}", e)
    };

    Ok(())
}
