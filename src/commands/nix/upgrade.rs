use crate::commands::nix::{nix_info, NIX_CMD, PROFILE_UPGRADE_ARGS, UNLOCKED_FLAKE_REF};
use anyhow::Result;
use clap::Parser;
use std::process::{Command, Stdio};
use tracing::info;

#[derive(Debug, Parser)]
pub struct NixUpgradeCommand {
    pub pkg: Option<String>,
}

pub fn nix_upgrade(command: NixUpgradeCommand) -> Result<()> {
    let output = if let Some(pkg) = command.pkg {
        info!("upgrading package {pkg}, this may take a while...");
        Command::new(NIX_CMD)
            .args(PROFILE_UPGRADE_ARGS)
            .arg(pkg.clone())
            .stdout(Stdio::inherit())
            .stderr(Stdio::null())
            .spawn()?
            .wait()?;
        // capture output of the command
        // because nix checks if the package is already up to date
        // this doesn't incur any extra overhead but
        // allows us to manage how errors are presented to users
        Command::new(NIX_CMD)
            .args(PROFILE_UPGRADE_ARGS)
            .arg(pkg)
            .output()?
    } else {
        info!("upgrading installed fuel.nix packages, this may take a while...");
        Command::new(NIX_CMD)
            .args(PROFILE_UPGRADE_ARGS)
            .arg(UNLOCKED_FLAKE_REF)
            .stdout(Stdio::inherit())
            .stderr(Stdio::null())
            .spawn()?
            .wait()?;
        Command::new(NIX_CMD)
            .args(PROFILE_UPGRADE_ARGS)
            .arg(UNLOCKED_FLAKE_REF)
            .output()?
    };

    nix_info!(output);

    Ok(())
}
