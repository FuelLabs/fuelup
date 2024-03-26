use anyhow::Result;
use std::str::FromStr;

use crate::{
    commands::toolchain::UninstallCommand,
    fmt::{println_error, println_warn},
    toolchain::{DistToolchainDescription, Toolchain},
};

pub fn uninstall(command: UninstallCommand) -> Result<()> {
    let UninstallCommand { name } = command;

    let toolchain = match DistToolchainDescription::from_str(&name) {
        Ok(desc) => Toolchain::from_path(&desc.to_string()),
        Err(_) => Toolchain::from_path(&name),
    };

    if !toolchain.exists() {
        println_warn(format!("Toolchain '{}' does not exist", &toolchain.name));
        return Ok(());
    }

    let active_toolchain = Toolchain::from_settings()?;
    if active_toolchain.name == toolchain.name {
        println_error(format!("Cannot uninstall '{}' as it is currently the default toolchain. Run `fuelup default <toolchain>` to update the default toolchain.", &toolchain.name));
        return Ok(());
    }

    match toolchain.uninstall_self() {
        Ok(_) => println!("Toolchain '{}' uninstalled", &toolchain.name),
        Err(e) => println_error(format!(
            "Failed to uninstall toolchain '{}': {}",
            &toolchain.name, e
        )),
    };

    Ok(())
}
