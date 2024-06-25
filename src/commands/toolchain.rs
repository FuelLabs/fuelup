use crate::ops::fuelup_toolchain::{install::install, new::new, uninstall::uninstall};
use crate::target_triple::TargetTriple;
use crate::toolchain::RESERVED_TOOLCHAIN_NAMES;
use anyhow::{bail, Result};
use clap::Parser;

#[derive(Debug, Parser)]
pub enum ToolchainCommand {
    /// Install or update a distributable toolchain
    Install(InstallCommand),
    /// Create a new custom toolchain
    New(NewCommand),
    /// Uninstall a toolchain
    Uninstall(UninstallCommand),
}

#[derive(Debug, Parser)]
pub struct InstallCommand {
    /// Toolchain name [possible values: latest, nightly, testnet, beta-1, beta-2, beta-3, beta-4, beta-5]
    pub name: String,
}

#[derive(Debug, Parser)]
pub struct NewCommand {
    /// Custom toolchain name. Names starting with distributable toolchain names are not allowed.
    #[clap(value_parser = name_allowed)]
    pub name: String,
}

#[derive(Debug, Parser)]
pub struct UninstallCommand {
    /// Toolchain to uninstall
    pub name: String,
}

fn name_allowed(s: &str) -> Result<String> {
    let name = match s.split_once('-') {
        Some((prefix, target_triple)) => {
            if TargetTriple::from_host()?.to_string() == target_triple {
                prefix
            } else {
                s
            }
        }
        None => s,
    };

    if RESERVED_TOOLCHAIN_NAMES.contains(&name) {
        bail!(
            "Cannot use distributable toolchain name '{}' as a custom toolchain name",
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
        ToolchainCommand::Uninstall(command) => uninstall(command)?,
    };

    Ok(())
}
