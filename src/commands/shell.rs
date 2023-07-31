use anyhow::{bail, Result};
use clap::Parser;
use std::process::Command;
use tracing::info;
use std::io::{self, Write};

use crate::ops::fuelup_toolchain::install::{FUEL_NIX_LINK, NIX_CMD};

use super::toolchain::{NeedsNix, NixName};


#[derive(Debug, Parser)]
pub struct ShellCommand {
    /// Open a new bash development shell with specified toolchain.
    pub toolchain: String,
}
impl NeedsNix for ShellCommand {
    fn get_toolchain(&self) -> &str {
        self.toolchain.as_str()
    }
}
impl NixName for ShellCommand {}
const SHELL: &str = "shell";
pub fn exec(command: ShellCommand) -> Result<()> {
    info!(
        "starting new bash shell with {} toolchain available on $PATH...",
        command.toolchain
    );
    let shell_cmd = format!("{NIX_CMD} {SHELL} {}", command.toolchain_link()?);
    if let Ok(mut child) = Command::new(NIX_CMD).arg(SHELL).arg(command.toolchain_link()?)
        .spawn() {
            child.wait()?;
        }
    
    Ok(())
}
