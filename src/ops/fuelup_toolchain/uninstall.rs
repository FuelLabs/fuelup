use anyhow::Result;
use std::fs;
use std::str::FromStr;
use tracing::info;

use crate::{
    commands::toolchain::UninstallCommand,
    config::Config,
    toolchain::{OfficialToolchainDescription, Toolchain},
};

pub fn uninstall(command: UninstallCommand) -> Result<()> {
    let UninstallCommand { name } = command;

    let mut toolchain = Toolchain::from(&name)?;

    if toolchain.is_official() {
        let description = OfficialToolchainDescription::from_str(&name)?;
        toolchain = Toolchain::from(&description.to_string())?;

        let config = Config::from_env()?;

        if config.hash_exists(&description) {
            let hash_file = config.hashes_dir().join(description.to_string());
            fs::remove_file(hash_file)?;
        };
    }

    if !toolchain.exists() {
        info!("toolchain '{}' does not exist", &toolchain.name);
        return Ok(());
    }

    toolchain.uninstall_self()?;
    info!("toolchain '{}' uninstalled", &toolchain.name);

    Ok(())
}
