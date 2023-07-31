use crate::commands::toolchain::{InstallCommand, NixName};
use anyhow::{bail, Result};
use std::process::Command;
use tracing::info;

pub(crate) const NIX_CMD: &str = "nix";
const PROFILE_INSTALL: &[&str; 2] = &["profile", "install"];
pub(crate) const FUEL_NIX_LINK: &str = "github:fuellabs/fuel.nix";

pub fn install(command: InstallCommand) -> Result<()> {
    info!(
        "downloading and installing {} toolchain, if this is the first time it may take a while...",
        command.name
    );
    if let Err(err) = Command::new(NIX_CMD)
        .args(PROFILE_INSTALL)
        .arg(command.toolchain_link()?)
        .output()
    {
        bail!("failed to install {} toolchain: {err}", command.name)
    }
    Ok(())
}
