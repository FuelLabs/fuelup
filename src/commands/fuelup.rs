use anyhow::Result;
use clap::Parser;

use crate::ops::fuelup_self::self_update;

#[derive(Debug, Parser)]
pub enum FuelupCommand {
    /// Updates fuelup
    Update,
}

#[derive(Debug, Parser)]
struct UpdateCommand {}

pub fn exec() -> Result<()> {
    self_update()?;

    Ok(())
}
