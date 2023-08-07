use crate::commands::nix::{
    link_generator::{NeedsNix, NixName},
    nix_info, NIX_CMD, PRIORITY_FLAG, PROFILE_INSTALL_ARGS,
};
use anyhow::{anyhow, bail, Result};
use clap::Parser;
use std::process::Command;
use tracing::info;

#[derive(Debug, Parser)]
pub struct NixInstallCommand {
    /// Toolchain or component
    pub name: String,
    pub priority: Option<String>,
}

pub fn nix_install(command: NixInstallCommand) -> Result<()> {
    let (output, priority) = if command.is_toolchain() {
        info!(
            "downloading and installing fuel {} toolchain, this may take a while...",
            command.name
        );
        let (output, priority) = if let Some(ref priority) = command.priority {
            let output = Command::new(NIX_CMD)
                .args(PROFILE_INSTALL_ARGS)
                .arg(command.toolchain_link()?)
                .arg(PRIORITY_FLAG)
                .arg(priority)
                .output()
                .map_err(|err| {
                    anyhow!("failed to install fuel {} toolchain: {err}", command.name)
                })?;
            (output, Some(priority))
        } else {
            let output = Command::new(NIX_CMD)
                .args(PROFILE_INSTALL_ARGS)
                .arg(command.toolchain_link()?)
                .output()
                .map_err(|err| {
                    anyhow!("failed to install fuel {} toolchain: {err}", command.name)
                })?;
            (output, None)
        };
        (output, priority)
    } else if command.is_component() {
        info!(
            "downloading and installing {} component, this may take a while...",
            command.name
        );
        let (output, priority) = if let Some(ref priority) = command.priority {
            let output = Command::new(NIX_CMD)
                .args(PROFILE_INSTALL_ARGS)
                .arg(command.component_link()?)
                .arg(PRIORITY_FLAG)
                .arg(priority)
                .output()
                .map_err(|err| anyhow!("failed to install {} component: {err}", command.name))?;
            (output, Some(priority))
        } else {
            let output = Command::new(NIX_CMD)
                .args(PROFILE_INSTALL_ARGS)
                .arg(command.component_link()?)
                .output()
                .map_err(|err| anyhow!("failed to install {} component: {err}", command.name))?;
            (output, None)
        };
        (output, priority)
    } else {
        bail!(
            "available distrubuted components:\n  -fuel-core\n  -fuel-core-client\n  -fuel-indexer\n  -forc\n  -forc-client\n  -forc-doc\n  -forc-explore\n  -forc-fmt\n  -forc-index\n  -forc-lsp\n  -forc-tx\n  -forc-wallet\n  -sway-vim\n
available distributed toolchains:\n  -latest\n  -nightly\n  -beta-1\n  -beta-2\n  -beta-3\n  -beta-4-rc

please form a valid component or toolchain, like so: fuel-core-beta-3 or beta-3"
        )
    };

    nix_info!(output);

    if priority.is_some() && output.stderr.is_empty() {
        info!(
            "successfully added {} with priority {}",
            command.name,
            priority.unwrap()
        );
    } else {
        info!("successfully added {}", command.name,);
    }

    Ok(())
}
