use crate::ops::fuelup_default;
use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
pub struct DefaultCommand {
    /// Set the default toolchain.
    pub toolchain: Option<String>,
}

pub fn exec(command: DefaultCommand) -> Result<()> {
    let DefaultCommand { toolchain } = command;
    fuelup_default::default(toolchain)
}
