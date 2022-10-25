use anyhow::Result;
use clap::Parser;

use crate::ops::fuelup_update;

#[derive(Debug, Parser)]
pub struct UpdateCommand {}

pub fn exec() -> Result<()> {
    fuelup_update::update()?;

    Ok(())
}
