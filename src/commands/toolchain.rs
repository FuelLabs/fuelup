use anyhow::{bail, Result};
use clap::Parser;

use crate::ops::fuelup_toolchain::install::install;
use crate::ops::fuelup_toolchain::install::FUEL_NIX_LINK;
use crate::ops::fuelup_toolchain::list_revisions::list_revisions;
use crate::ops::fuelup_toolchain::new::new;
use crate::ops::fuelup_toolchain::uninstall::uninstall;
use crate::target_triple::TargetTriple;
use crate::toolchain::RESERVED_TOOLCHAIN_NAMES;

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
}
pub trait NeedsNix {
    fn get_toolchain(&self) -> &str;
}
pub trait NixName: NeedsNix {
    fn nix_suffix(&self) -> Result<&str> {
        let suffix = match self.get_toolchain() {
                "latest" => "fuel",
                "nightly" => "fuel-nightly",
                "beta-1" | "beta1" => "fuel-beta-1",
                "beta-2" | "beta2" => "fuel-beta-2",
                "beta-3" | "beta3" => "fuel-beta-3",
                "beta-4-rc" | "beta-4rc" | "beta4rc" => "fuel-beta-4-rc",
                _ => bail!("available toolchains:\n  -latest\n  -nightly\n  -beta-1\n  -beta-2\n  -beta-3\n  -beta-4-rc")
            };
        Ok(suffix)
    }
    fn toolchain_link(&self) -> Result<String> {
        Ok(format!("{FUEL_NIX_LINK}#{}", self.nix_suffix()?))
    }
}

#[derive(Debug, Parser)]
pub struct InstallCommand {
    /// Toolchain name [possible values: latest, beta-1, beta-2, beta-3, beta-4-rc, nightly]
    pub name: String,
}
impl NeedsNix for InstallCommand {
    fn get_toolchain(&self) -> &str {
        self.name.as_str()
    }
}
impl NixName for InstallCommand {}

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
        ToolchainCommand::ListRevisions(command) => list_revisions(command)?,
    };

    Ok(())
}
