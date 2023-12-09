use anyhow::{bail, Ok, Result};
use clap::Parser;
use serde::Deserialize;
use std::io::stdin;
use std::str::FromStr;

use crate::constants::VALID_CHANNEL_STR;
use crate::ops::fuelup_toolchain::export::export;
use crate::ops::fuelup_toolchain::install::install;
use crate::ops::fuelup_toolchain::list_revisions::list_revisions;
use crate::ops::fuelup_toolchain::new::new;
use crate::ops::fuelup_toolchain::uninstall::uninstall;
use crate::target_triple::TargetTriple;
use crate::toolchain::RESERVED_TOOLCHAIN_NAMES;
use crate::toolchain_override;

#[derive(Debug, Parser)]
pub enum ToolchainCommand {
    /// Install or update a distributable toolchain
    Install(InstallCommand),
    /// Create a new custom toolchain
    New(NewCommand),
    /// Uninstall a toolchain
    Uninstall(UninstallCommand),
    /// Fetch the list of published `latest` toolchains, starting from the most recent
    ListRevisions(ListRevisionsCommand),
    /// Export the toolchain info into fuel-toolchain.toml
    Export(ExportCommand),
}

#[derive(Debug, Parser)]
pub struct InstallCommand {
    /// Toolchain name [possible values: latest, beta-1, beta-2, beta-3, beta-4, nightly]
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

#[derive(Debug, Parser)]
pub struct ListRevisionsCommand {}

#[derive(Debug, Deserialize, Parser)]
pub struct ExportCommand {
    /// Toolchain to export, [possible values: latest, beta-1, beta-2, beta-3, beta-4, nightly].
    /// The default toolchain will be exported if name isn't specified
    pub name: Option<String>,
    /// Channel to export, [possible values: latest-YYYY-MM-DD, nightly-YYYY-MM-DD, beta-1, beta-2, beta-3, beta-4].
    #[clap(value_parser = channel_allowed)]
    pub channel: Option<String>,
    /// Forces exporting the toolchain, replacing any existing toolchain override file
    #[arg(short, long)]
    pub force: bool,
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

fn channel_allowed(s: &str) -> Result<Option<String>> {
    if s.is_empty() {
        return Ok(None);
    }
    if toolchain_override::Channel::from_str(s).is_err() {
        bail!(
            "Invalid channel '{}', expected one of {}.",
            s,
            VALID_CHANNEL_STR,
        );
    }
    Ok(Some(s.to_string()))
}

pub fn exec(command: ToolchainCommand) -> Result<()> {
    match command {
        ToolchainCommand::Install(command) => install(command)?,
        ToolchainCommand::New(command) => new(command)?,
        ToolchainCommand::Uninstall(command) => uninstall(command)?,
        ToolchainCommand::ListRevisions(command) => list_revisions(command)?,
        ToolchainCommand::Export(command) => export(command, stdin().lock())?,
    };

    Ok(())
}
