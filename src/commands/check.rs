use anyhow::Result;
use clap::Parser;

use crate::ops::fuelup_check;

#[derive(Debug, Parser)]
pub struct CheckCommand {
    /// Whether to explicitly show versioning of forc plugins, which is normally shown together
    /// with forc.
    #[clap(long)]
    pub verbose: bool,
}

pub fn exec(command: CheckCommand) -> Result<()> {
    fuelup_check::check(command)?;

    Ok(())
}
