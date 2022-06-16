use anyhow::{bail, Result};
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
    if let Err(e) = self_update() {
        bail!("fuelup failed to update itself: {}", e)
    };

    Ok(())
}
