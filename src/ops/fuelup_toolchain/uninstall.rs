use anyhow::{bail, Result};
use std::fs;
use std::str::FromStr;
use tracing::{error, info};

use crate::{
    commands::toolchain::UninstallCommand,
    config::Config,
    ops::fuelup_default,
    toolchain::{DistToolchainDescription, Toolchain},
};

pub fn uninstall(command: UninstallCommand) -> Result<()> {
    let UninstallCommand { name } = command;

    let config = Config::from_env()?;

    let toolchain = match DistToolchainDescription::from_str(&name) {
        Ok(desc) => {
            if config.hash_exists(&desc) {
                let hash_file = config.hashes_dir().join(desc.to_string());
                fs::remove_file(hash_file)?;
            };

            Toolchain::from_path(&desc.to_string())
        }
        Err(_) => Toolchain::from_path(&name),
    };

    if !toolchain.exists() {
        info!("toolchain '{}' does not exist", &toolchain.name);
        return Ok(());
    }

    match toolchain.uninstall_self() {
        Ok(_) => {
            info!("toolchain '{}' uninstalled", &toolchain.name);
            let active_toolchain = Toolchain::from_settings()?;
            if active_toolchain.name == toolchain.name {
                for toolchain in config.list_toolchains()? {
                    if fuelup_default::default(Some(toolchain)).is_ok() {
                        return Ok(());
                    }
                }

                error!(
                "Could not set default toolchain after uninstallation of currently used toolchain. 
                Please run `fuelup default <toolchain>` to manually switch your current toolchain."
                )
            }
        }
        Err(e) => {
            bail!("Failed to uninstall toolchain '{}': {}", &toolchain.name, e)
        }
    };

    Ok(())
}
