use anyhow::Result;
use clap::Parser;

use crate::ops::fuelup_toolchain;

#[derive(Debug, Parser)]
pub enum ToolchainCommand {
    /// Install or update a given toolchain
    ///
    /// Currently, we only support installation of the 'latest' toolchain:
    /// `fuelup toolchain install latest`
    Install(InstallCommand),
}

#[derive(Debug, Parser)]
pub struct InstallCommand {
    /// Toolchain name [possible values: latest]
    pub name: String,
}

pub fn exec(command: ToolchainCommand) -> Result<()> {
    match command {
        ToolchainCommand::Install(command) => fuelup_toolchain::install(command)?,
    };

    Ok(())
}
