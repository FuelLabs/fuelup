use anyhow::Result;
use clap::Parser;

use crate::ops::fuelup_check;

#[derive(Debug, Parser)]
pub struct CheckCommand {}

pub fn exec() -> Result<()> {
    fuelup_check::check()?;

    Ok(())
}
