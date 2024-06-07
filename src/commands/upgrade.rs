use anyhow::Result;
use clap::Parser;

use crate::ops::fuelup_upgrade;

#[derive(Debug, Parser)]
pub struct UpgradeCommand {
    #[clap(long, short)]
    pub force: bool,
}

pub fn exec(force: bool) -> Result<()> {
    fuelup_upgrade::upgrade(force)?;

    Ok(())
}
