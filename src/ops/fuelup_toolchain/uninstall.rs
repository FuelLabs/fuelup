use anyhow::{bail, Result};
use std::str::FromStr;

use crate::{
    commands::toolchain::UninstallCommand,
    config::Config,
    fmt::{println_error, println_warn},
    ops::fuelup_default,
    toolchain::{DistToolchainDescription, Toolchain},
};

pub fn uninstall(command: UninstallCommand) -> Result<()> {
    let UninstallCommand { name } = command;

    let config = Config::from_env()?;

    let toolchain = match DistToolchainDescription::from_str(&name) {
        Ok(desc) => Toolchain::from_path(&desc.to_string()),
        Err(_) => Toolchain::from_path(&name),
    };

    if !toolchain.exists() {
        println_warn(format!("Toolchain '{}' does not exist", &toolchain.name));
        return Ok(());
    }

    if config.list_toolchains()?.len() == 1 {
        bail!("Cannot uninstall the last toolchain");
    }

    match toolchain.uninstall_self() {
        Ok(_) => {
            println!("Toolchain '{}' uninstalled", &toolchain.name);
            let active_toolchain = Toolchain::from_settings()?;
            if active_toolchain.name == toolchain.name {
                for toolchain in config.list_toolchains()? {
                    if fuelup_default::default(Some(toolchain)).is_ok() {
                        return Ok(());
                    }
                }

                println_error(format!(
                    "{}\r\t{}",
                    "Could not set default toolchain after uninstallation of currently used toolchain",
                    "Please run `fuelup default <toolchain>` to manually switch your current toolchain."
                ));
            }
        }
        Err(e) => println_error(format!(
            "Failed to uninstall toolchain '{}': {}",
            &toolchain.name, e
        )),
    };

    Ok(())
}
