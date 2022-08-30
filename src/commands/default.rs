use std::str::FromStr;

use crate::toolchain::OfficialToolchainDescription;
use anyhow::{bail, Result};
use clap::Parser;
use tracing::info;

use crate::{path::settings_file, settings::SettingsFile, toolchain::Toolchain};

#[derive(Debug, Parser)]
pub struct DefaultCommand {
    /// Set the default toolchain.
    pub toolchain: Option<String>,
}

pub fn exec(command: DefaultCommand) -> Result<()> {
    let DefaultCommand { toolchain } = command;

    let current_toolchain = Toolchain::from_settings()?;

    let toolchain = match toolchain {
        Some(toolchain) => toolchain,
        None => {
            info!("{} (default)", current_toolchain.name);
            return Ok(());
        }
    };

    let mut new_default = Toolchain::from(&toolchain)?;

    if OfficialToolchainDescription::from_str(&toolchain).is_ok() {
        new_default = Toolchain::new(&toolchain)?;
    } else if !new_default.exists() {
        bail!("Toolchain with name '{}' does not exist", &new_default.name);
    };

    let settings = SettingsFile::new(settings_file());
    settings.with_mut(|s| {
        s.default_toolchain = Some(new_default.name.clone());
        Ok(())
    })?;
    info!("default toolchain set to '{}'", new_default.name);

    Ok(())
}
