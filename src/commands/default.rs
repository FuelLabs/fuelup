use anyhow::Result;
use clap::Parser;

use crate::ops::fuelup_default;

#[derive(Debug, Parser)]
pub struct DefaultCommand {
    /// Set the default toolchain.
    pub toolchain: Option<String>,
}

pub fn exec(command: DefaultCommand) -> Result<()> {
    let DefaultCommand { toolchain } = command;

    fuelup_default::default(toolchain)
}
