use anyhow::{bail, Result};
use clap::{Args, Parser};

use crate::ops::fuelup_toolchain::install::{install, toolchain};
use crate::ops::fuelup_toolchain::new::new;

#[derive(Debug, Parser)]
pub enum ToolchainCommand {
    /// Install or update a given toolchain
    ///
    /// Currently, we only support installation of the 'latest' toolchain:
    /// `fuelup toolchain install latest`
    Install(InstallCommand),
    /// Create a new custom toolchain
    New(NewCommand),
}

#[derive(Debug, Parser)]
pub struct InstallCommand {
    /// Toolchain name [possible values: latest]
    pub name: String,
}

#[derive(Debug, Parser)]
pub struct NewCommand {
    /// Custom toolchain name. Names starting with 'latest' are not allowed.
    #[clap(value_parser = name_allowed)]
    pub name: String,
}

fn name_allowed(s: &str) -> Result<String> {
    if [toolchain::LATEST].contains(&s) {
        bail!(
            "Cannot use official toolchain name '{}' as a custom toolchain name",
            s
        )
    } else {
        Ok(s.to_string())
    }
}

pub fn exec(command: ToolchainCommand) -> Result<()> {
    match command {
        ToolchainCommand::Install(command) => install(command)?,
        ToolchainCommand::New(command) => new(command)?,
    };

    Ok(())
}
