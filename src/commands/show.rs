use anyhow::Result;
use clap::Parser;

use crate::ops::fuelup_show;

#[derive(Debug, Parser)]
pub struct ShowCommand {}

pub fn exec() -> Result<()> {
    fuelup_show::show()
}
