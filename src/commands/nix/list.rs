use super::{NIX_CMD, PROFILE_LIST_ARGS};
use crate::commands::nix::nix_info;
use anyhow::{bail, Result};
use clap::Parser;
use std::process::Command;
use tracing::info;

#[derive(Debug, Parser)]
pub struct NixListCommand;

pub fn nix_list(_command: NixListCommand) -> Result<()> {
    match Command::new(NIX_CMD).args(PROFILE_LIST_ARGS).output() {
        Ok(output) => {
            nix_info!(output);
            Ok(())
        }
        Err(err) => bail!("failed to show installed binaries for profile: {err}"),
    }
}
