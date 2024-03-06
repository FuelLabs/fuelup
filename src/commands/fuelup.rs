use anyhow::{bail, Result};
use clap::Parser;

use crate::ops::fuelup_self::self_update;

#[derive(Debug, Parser)]
pub enum FuelupCommand {
    /// Updates fuelup
    Update(UpdateCommand),
}

#[derive(Debug, Parser)]
pub struct UpdateCommand {
    #[clap(long, short)]
    pub force: bool,
}

pub fn exec(force: bool) -> Result<()> {
    if let Err(e) = self_update(force) {
        bail!("fuelup failed to update itself: {}", e)
    };

    Ok(())
}
