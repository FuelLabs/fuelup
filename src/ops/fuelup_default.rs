use anyhow::{bail, Result};
use std::str::FromStr;
use tracing::info;

use crate::{
    path::settings_file,
    settings::SettingsFile,
    toolchain::{DistToolchainDescription, Toolchain},
    toolchain_override::ToolchainOverride,
};

pub fn default(toolchain: Option<String>) -> Result<()> {
    let current_toolchain = Toolchain::from_settings()?;

    let toolchain = match toolchain {
        Some(toolchain) => toolchain,
        None => {
            let mut current_default = format!("{} (default)", current_toolchain.name);
            if let Some(to) = ToolchainOverride::from_file() {
                let name = match DistToolchainDescription::from_str(&to.toolchain.name) {
                    Ok(desc) => desc.to_string(),
                    Err(_) => to.toolchain.name,
                };
                current_default.push_str(&format!(", {} (override)", name))
            }

            info!("{}", current_default);
            return Ok(());
        }
    };

    let new_default = match DistToolchainDescription::from_str(&toolchain) {
        Ok(desc) => Toolchain::from_path(&desc.to_string())?,
        Err(_) => Toolchain::from_path(&toolchain)?,
    };

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
