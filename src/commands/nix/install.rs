use crate::commands::nix::{
    link_generator::{NeedsNix, NixName},
    NIX_CMD, PRIORITY_FLAG, PROFILE_INSTALL_ARGS,
};
use anyhow::{anyhow, bail, Result};
use clap::Parser;
use std::{
    fmt::Debug,
    process::{Command, Stdio},
    str::SplitWhitespace,
};
use tracing::info;

#[derive(Debug, Parser)]
pub struct NixInstallCommand {
    /// Toolchain or component
    pub name: String,
}

pub fn nix_install(command: NixInstallCommand) -> Result<()> {
    let (output, link) = if command.is_toolchain() {
        info!(
            "downloading and installing fuel {} toolchain, this may take a while...",
            command.name
        );
        let link = command.toolchain_link()?;
        let mut process = Command::new(NIX_CMD);
        process
            .args(PROFILE_INSTALL_ARGS)
            .arg(link.clone())
            .stdout(Stdio::inherit())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|err| anyhow!("failed to install fuel {} toolchain: {err}", command.name))?
            .wait()?;
        // second process is ran to get the output since we silence errors in the first
        // this doesn't incur any overhead since nix will look to see if the toolchain is already installed
        let mut process = Command::new(NIX_CMD);
        (
            process
                .args(PROFILE_INSTALL_ARGS)
                .arg(link.clone())
                .output()
                .unwrap(),
            link,
        )
    } else if command.is_component() {
        info!(
            "downloading and installing {} component, this may take a while...",
            command.name
        );
        let link = command.component_link()?;
        let mut process = Command::new(NIX_CMD);
        process
            .args(PROFILE_INSTALL_ARGS)
            .arg(link.clone())
            .stdout(Stdio::inherit())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|err| anyhow!("failed to install {} component: {err}", command.name))?
            .wait()?;
        (
            process
                .args(PROFILE_INSTALL_ARGS)
                .arg(link.clone())
                .output()
                .unwrap(),
            link,
        )
    } else {
        bail!(
            "available distrubuted components:\n  -fuel-core\n  -fuel-core-client\n  -fuel-indexer\n  -forc\n  -forc-client\n  -forc-doc\n  -forc-explore\n  -forc-fmt\n  -forc-index\n  -forc-lsp\n  -forc-tx\n  -forc-wallet\n  -sway-vim\n
available distributed toolchains:\n  -latest\n  -nightly\n  -beta-1\n  -beta-2\n  -beta-3\n  -beta-4-rc

please form a valid component or toolchain, like so: fuel-core-beta-3 or beta-3"
        )
    };

    // hacky way of getting the priority of the package automatically
    if !output.stderr.is_empty() {
        const NIX_PKG_MSG: &str = "The conflicting packages have a priority of";
        const NIXOS_MSG: &str = "have the same priority";
        let err_str = String::from_utf8_lossy(&output.stderr);
        // nix package manager
        if let Some(index) = err_str.find(NIX_PKG_MSG) {
            let (_first, last) = err_str.split_at(index);
            let iter = last.split_whitespace();
            auto_prioritize_installed_package(iter, 7, link)?;
        // nixos
        } else if let Some(index) = err_str.find(NIXOS_MSG) {
            let (_first, last) = err_str.split_at(index);
            let iter = last.split_whitespace();
            auto_prioritize_installed_package(iter, 4, link)?;
        }
    }

    // nix_info!(output);

    Ok(())
}

/// Given an iterator over a priority error message, get the priority for the installed packages
/// and prioritize the newly installed package.
///
/// This does not incur an overhead since nix will check if the package is already installed.
fn auto_prioritize_installed_package(
    mut iter: SplitWhitespace,
    len: usize,
    link: String,
) -> Result<()> {
    for _ in 0..len {
        iter.next();
    }
    if let Some(prio) = iter.next() {
        let chars = prio.chars();
        if let Ok(num) = chars
            .filter(|c| c.is_digit(10))
            .collect::<String>()
            .parse::<u32>()
        {
            Command::new(NIX_CMD)
                .args(PROFILE_INSTALL_ARGS)
                .arg(link)
                .arg(PRIORITY_FLAG)
                .arg((num - 1).to_string())
                .stdout(Stdio::inherit())
                .stderr(Stdio::null())
                .spawn()
                .map_err(|err| anyhow!("failed to auto set package priority: {err}"))?
                .wait()?;
        }
    }
    Ok(())
}
