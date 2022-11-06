use anyhow::{bail, Result};
use std::str::FromStr;
use tracing::info;

use crate::{
    path::settings_file,
    settings::SettingsFile,
    toolchain::{OfficialToolchainDescription, Toolchain},
};

pub fn default(toolchain: Option<String>) -> Result<()> {
    let current_toolchain = Toolchain::from_settings()?;

    let toolchain = match toolchain {
        Some(toolchain) => toolchain,
        None => {
            info!("{} (default)", current_toolchain.name);
            return Ok(());
        }
    };

    let mut new_default = Toolchain::from_path(&toolchain)?;

    if let Ok(description) = OfficialToolchainDescription::from_str(&toolchain) {
        new_default = Toolchain::from_path(&description.to_string())?;
    }

    if !new_default.exists() {
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
