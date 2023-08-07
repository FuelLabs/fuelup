use crate::commands::toolchain::{InstallCommand, NixName};
use anyhow::{anyhow, Result};
use std::process::Command;
use tracing::info;

pub(crate) const NIX_CMD: &str = "nix";
const PROFILE_INSTALL: &[&str; 2] = &["profile", "install"];
const PRIORITY: &str = "--priority";
pub(crate) const FUEL_NIX_LINK: &str = "github:fuellabs/fuel.nix";

pub fn install(command: InstallCommand) -> Result<()> {
    info!(
        "downloading and installing {} toolchain, if this is the first time it may take a while...",
        command.name
    );
    let (output, priority) = if let Some(ref priority) = command.priority {
        let output = Command::new(NIX_CMD)
            .args(PROFILE_INSTALL)
            .arg(command.toolchain_link()?)
            .arg(PRIORITY)
            .arg(priority)
            .output()
            .map_err(|err| anyhow!("failed to install {} toolchain: {err}", command.name))?;
        (output, Some(priority))
    } else {
        let output = Command::new(NIX_CMD)
            .args(PROFILE_INSTALL)
            .arg(command.toolchain_link()?)
            .output()
            .map_err(|err| anyhow!("failed to install {} toolchain: {err}", command.name))?;
        (output, None)
    };
    if !output.stdout.is_empty() {
        info!("{}", String::from_utf8_lossy(&output.stdout));
    }
    if !output.stderr.is_empty() {
        info!("{}", String::from_utf8_lossy(&output.stderr));
    }

    if priority.is_some() {
        info!(
            "successfully added {} with priority {}",
            command.name,
            priority.unwrap()
        );
    }

    Ok(())
}
