use crate::commands::nix::{nix_info, NIX_CMD, PROFILE_REMOVE_ARGS};
use anyhow::Result;
use clap::Parser;
use std::process::Command;
use tracing::info;

#[derive(Debug, Parser)]
pub struct NixRemoveCommand {
    pub pkg: String,
}

pub fn nix_remove(command: NixRemoveCommand) -> Result<()> {
    let output = Command::new(NIX_CMD)
        .args(PROFILE_REMOVE_ARGS)
        .arg(command.pkg)
        .output()?;

    nix_info!(output);

    Ok(())
}
