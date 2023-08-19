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

#[derive(Debug, Parser)]
pub struct NixInstallCommand {
    /// Toolchain or component
    pub name: String,
}

pub fn nix_install(command: NixInstallCommand) -> Result<()> {
    let (tx, rx) = mpsc::channel();
    let (err_str, link) = if command.is_toolchain() {
        info!(
            "downloading and installing fuel {} toolchain, this may take a while...",
            command.name
        );
        let link = command.toolchain_link()?;

        let link_clone = link.clone();
        let command_name = command.name.clone();
        let mut err_lines = Vec::new();
        if let Ok(mut child) = Command::new(NIX_CMD)
            .args(PROFILE_INSTALL_ARGS)
            .arg(link_clone)
            .arg("--log-format")
            .arg("bar-with-logs")
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|err| anyhow!("failed to install fuel {} toolchain: {err}", command_name))
        {
            let handle = thread::spawn(move || {
                while let Some(stderr) = child.stderr.take() {
                    let reader = BufReader::new(stderr);

                    let mut contains_err = false;

                    for line in reader.lines() {
                        if let Ok(line) = line {
                            if !line.contains("error") && !contains_err {
                                tx.send((Some(line), None)).unwrap();
                            } else {
                                contains_err = true;
                                tx.send((None, Some(line))).unwrap();
                            }
                        }
                    }
                }
            });

            while let Ok((line, err)) = rx.recv() {
                if let Some(line) = line {
                    info!("AYO: {line}");
                }
                if let Some(err) = err {
                    err_lines.push(err);
                }
            }
            handle.join().unwrap();
        }

        (err_lines.concat(), link)
    } else if command.is_component() {
        info!(
            "downloading and installing {} component, this may take a while...",
            command.name
        );
        let link = command.component_link()?;

        let link_clone = link.clone();
        let command_name = command.name.clone();

        let handle = thread::spawn(move || {
            if let Ok(mut child) = Command::new(NIX_CMD)
                .args(PROFILE_INSTALL_ARGS)
                .arg(link_clone)
                .stderr(Stdio::piped())
                .spawn()
                .map_err(|err| anyhow!("failed to install {} component: {err}", command_name))
            {
                while let Some(stderr) = child.stderr.take() {
                    let reader = BufReader::new(stderr);

                    let mut contains_err = false;

                    for line in reader.lines() {
                        if let Ok(line) = line {
                            if !line.contains("error") && !contains_err {
                                tx.send((Some(line), None)).unwrap();
                            } else {
                                contains_err = true;
                                tx.send((None, Some(line))).unwrap();
                            }
                        }
                    }
                }
            }
        });

        let mut err_lines = Vec::new();
        while let Ok((line, err)) = rx.recv() {
            if let Some(line) = line {
                info!("{line}");
            }
            if let Some(err) = err {
                err_lines.push(err);
            }
        }
        handle.join().unwrap();

        (err_lines.concat(), link)
    } else {
        bail!(
            "available distrubuted components:\n  -fuel-core\n  -fuel-core-client\n  -fuel-indexer\n  -forc\n  -forc-client\n  -forc-doc\n  -forc-explore\n  -forc-fmt\n  -forc-index\n  -forc-lsp\n  -forc-tx\n  -forc-wallet\n  -sway-vim\n
available distributed toolchains:\n  -latest\n  -nightly\n  -beta-1\n  -beta-2\n  -beta-3\n  -beta-4-rc

please form a valid component or toolchain, like so: fuel-core-beta-3 or beta-3"
        )
    };

    // hacky way of getting the priority of the package automatically
    if !err_str.is_empty() {
        const NIX_PKG_MSG: &str = "The conflicting packages have a priority of";
        const NIXOS_MSG: &str = "have the same priority";
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
            .filter(|c| c.is_ascii_digit())
            .collect::<String>()
            .parse::<u32>()
        {
            // increase the number for the priority of the pkg that is older
            // if we decrease the priority for the new package, eventually
            // we could end up with collisions were two packages have the highest priority (0)
            println!("cmd: nix profile install --priority {} {}", num - 1, link);
            Command::new(NIX_CMD)
                .args(PROFILE_INSTALL_ARGS)
                .arg(PRIORITY_FLAG)
                .arg((num - 1).to_string())
                .arg(link)
                // .stderr(Stdio::null())
                .spawn()
                .map_err(|err| anyhow!("failed to auto set package priority: {err}"))?
                .wait()?;
        }
    }
    Ok(())
}
