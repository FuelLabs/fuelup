use crate::commands::nix::{
    link_generator::{NeedsNix, NixName},
    NIX_CMD, PRIORITY_FLAG, PROFILE_INSTALL_ARGS,
};
use anyhow::{anyhow, bail, Result};
use clap::Parser;
use std::{
    fmt::Debug,
    io::{BufRead, BufReader},
    process::{Command, Stdio},
    str::SplitWhitespace,
    sync::mpsc,
    thread,
};
use tracing::info;

const NIX_PKG_PRIORITY_MSG: &str = "The conflicting packages have a priority of";
const NIXOS_PRIORITY_MSG: &str = "have the same priority";

#[derive(Debug, Parser)]
pub struct NixInstallCommand {
    /// Toolchain or component
    pub name: String,
}

pub fn nix_install(command: NixInstallCommand) -> Result<()> {
    let (priority_err, link) = if command.is_toolchain() {
        info!(
            "downloading and installing fuel {} toolchain, this may take a while...",
            command.name
        );
        let link = command.toolchain_link()?;
        let mut priority_err = Vec::new();
        filter_command(link.clone(), command.name.clone(), &mut priority_err, true);

        (priority_err.concat(), link)
    } else if command.is_component() {
        info!(
            "downloading and installing component {}, this may take a while...",
            command.name
        );
        let link = command.component_link()?;
        let mut priority_err = Vec::new();
        filter_command(link.clone(), command.name.clone(), &mut priority_err, false);

        (priority_err.concat(), link)
    } else {
        bail!(
            "available distrubuted components:\n  -fuel-core\n  -fuel-core-client\n  -fuel-indexer\n  -forc\n  -forc-client\n  -forc-doc\n  -forc-explore\n  -forc-fmt\n  -forc-index\n  -forc-lsp\n  -forc-tx\n  -forc-wallet\n  -sway-vim\n
available distributed toolchains:\n  -latest\n  -nightly\n  -beta-1\n  -beta-2\n  -beta-3\n  -beta-4-rc

please form a valid component or toolchain, like so: fuel-core-beta-3 or beta-3"
        )
    };

    // hacky way of getting the priority of the package automatically
    if !priority_err.is_empty() {
        // nix package manager
        if let Some(index) = priority_err.find(NIX_PKG_PRIORITY_MSG) {
            let (_, err) = priority_err.split_at(index);
            let iter = err.split_whitespace();
            auto_prioritize_installed_package(iter, 7, link)?;
        // nixos
        } else if let Some(index) = priority_err.find(NIXOS_PRIORITY_MSG) {
            let (_, err) = priority_err.split_at(index);
            let iter = err.split_whitespace();
            auto_prioritize_installed_package(iter, 4, link)?;
        }
    }

    Ok(())
}

/// Execute the `Command` and filter the priority errors so we can handle it for the user automatically.
fn filter_command(
    link_clone: String,
    command_name: String,
    priority_err: &mut Vec<String>,
    is_toolchain_cmd: bool,
) {
    let (tx, rx) = mpsc::channel();
    if let Ok(mut child) = Command::new(NIX_CMD)
        .args(PROFILE_INSTALL_ARGS)
        .arg(link_clone)
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| {
            if is_toolchain_cmd {
                anyhow!("failed to install fuel {} toolchain: {err}", command_name)
            } else {
                anyhow!("failed to install component {}: {err}", command_name)
            }
        })
    {
        let handle = thread::spawn(move || {
            while let Some(stderr) = child.stderr.take() {
                let reader = BufReader::new(stderr);

                for line in reader.lines() {
                    if let Ok(line) = line {
                        if line.contains(NIXOS_PRIORITY_MSG) || line.contains(NIX_PKG_PRIORITY_MSG)
                        {
                            tx.send((None, Some(line))).unwrap();
                        } else {
                            tx.send((Some(line), None)).unwrap();
                        }
                    }
                }
            }
        });

        while let Ok((line, err)) = rx.recv() {
            if let Some(line) = line {
                info!("{line}");
            }
            if let Some(err) = err {
                priority_err.push(err);
            }
        }
        handle.join().unwrap();
    }
}

/// Given an iterator over a priority error message, get the priority for the installed packages
/// and prioritize the newly installed package.
///
/// This does not incur an overhead since nix will check if the package is already installed.
fn auto_prioritize_installed_package(
    mut iter: SplitWhitespace,
    msg_len: usize,
    link: String,
) -> Result<()> {
    for _ in 0..msg_len {
        iter.next();
    }
    if let Some(given_priority) = iter.next() {
        let chars = given_priority.chars();
        if let Ok(current_pkg_priority) = chars
            .filter(|c| c.is_ascii_digit())
            .collect::<String>()
            .parse::<i32>()
        {
            try_prioritize(current_pkg_priority, link)?
        }
    }
    Ok(())
}

/// `nix profile install --priority` can be negative, so here we just continue to try
/// installing the package with decreasing priority number until the error goes away.
///
/// There currently isn't a way to check the priority of packages other than the error
/// provided by nix when installing a package that it finds a conflict with.
fn try_prioritize(mut pkg_priority: i32, link: String) -> Result<()> {
    pkg_priority -= 1;
    let output = Command::new(NIX_CMD)
        .args(PROFILE_INSTALL_ARGS)
        .arg(PRIORITY_FLAG)
        .arg(pkg_priority.to_string())
        .arg(link.clone())
        .output()?;
    if !output.stderr.is_empty() {
        let stderr_str = String::from_utf8_lossy(&output.stderr);
        if stderr_str.contains(NIXOS_PRIORITY_MSG) || stderr_str.contains(NIX_PKG_PRIORITY_MSG) {
            // recursively decriment the package priority until the
            // newly installed package has the highest priority
            try_prioritize(pkg_priority, link)?
        }
    }
    Ok(())
}
