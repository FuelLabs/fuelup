use crate::commands::nix::{nix_info, NIX_CMD, PROFILE_UPGRADE_ARGS, UNLOCKED_FLAKE_REF};
use anyhow::Result;
use clap::Parser;
use std::process::Command;
use tracing::info;

#[derive(Debug, Parser)]
pub struct NixUpgradeCommand {
    pub pkg: Option<String>,
}

pub fn nix_upgrade(command: NixUpgradeCommand) -> Result<()> {
    let output = if let Some(pkg) = command.pkg {
        info!("upgrading installed fuel.nix package, this may take a while...");
        Command::new(NIX_CMD)
            .args(PROFILE_UPGRADE_ARGS)
            .arg(pkg)
            .output()?
    } else {
        info!("upgrading installed fuel.nix packages, this may take a while...");
        Command::new(NIX_CMD)
            .args(PROFILE_UPGRADE_ARGS)
            .arg(UNLOCKED_FLAKE_REF)
            .output()?
    };

    nix_info!(output);

    Ok(())
}
