use anyhow::Result;
use clap::Parser;

use crate::ops::fuelup_update;

#[derive(Debug, Parser)]
pub struct UpdateCommand {}

pub fn exec(command: UpdateCommand) -> Result<()> {
    fuelup_update::update(command)?;

    Ok(())
}
