use crate::path::{ensure_dir_exists, toolchain_bin_dir};
use crate::{commands::toolchain::NewCommand, path::toolchain_dir};
use anyhow::bail;
use anyhow::Result;
use std::fs;
use std::io;

pub fn new(command: NewCommand) -> Result<()> {
    let NewCommand { name } = command;

    let toolchain_dir = toolchain_dir();

    let toolchains: Vec<String> = fs::read_dir(&toolchain_dir)?
        .filter_map(io::Result::ok)
        .filter(|e| e.file_type().map(|f| f.is_dir()).unwrap_or(false))
        .map(|e| e.file_name().into_string().ok().unwrap_or_default())
        .collect();

    if toolchains.contains(&name) {
        bail!("Toolchain with name '{}' already exists", &name)
    }

    let toolchain_bin_dir = toolchain_bin_dir(&name);

    if ensure_dir_exists(&toolchain_dir.join(toolchain_bin_dir)).is_ok() {
        println!("New toolchain initialized: {}", &name);
    };

    Ok(())
}
