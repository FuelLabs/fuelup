use crate::{
    config,
    fmt::print_header,
    path::settings_file,
    settings::SettingsFile,
    toolchain::{DistToolchainDescription, Toolchain},
    toolchain_override::ToolchainOverride,
};
use anyhow::{bail, Result};
use std::str::FromStr;
use tracing::info;

pub fn default(toolchain: Option<String>) -> Result<()> {
    let toolchain = if let Some(toolchain) = toolchain {
        toolchain
    } else {
        let mut result = String::new();
        let current_toolchain = Toolchain::from_settings()?;
        if let Some(to) = ToolchainOverride::from_project_root() {
            let name =
                match DistToolchainDescription::from_str(&to.cfg.toolchain.channel.to_string()) {
                    Ok(desc) => desc.to_string(),
                    Err(_) => to.cfg.toolchain.channel.to_string(),
                };
            result.push_str(&format!("{name} (override)"));

            if current_toolchain.exists() {
                result.push_str(", ");
            }
        }
        result.push_str(&format!("{} (default)", current_toolchain.name));
        info!("{}", result);
        return Ok(());
    };

    let new_default = match DistToolchainDescription::from_str(&toolchain) {
        Ok(desc) => Toolchain::from_path(&desc.to_string()),
        Err(_) => Toolchain::from_path(&toolchain),
    };
    if !new_default.exists() {
        let cfg = config::Config::from_env()?;
        let toolchains = cfg.list_toolchains()?;

        info!("Toolchain with name '{}' does not exist", &new_default.name);
        print_header("Installed toolchains");
        for toolchain in toolchains {
            info!("{}", toolchain);
        }
        // TODO: we should consider migrating to use thiserror in order to return a custom error here
        // so we can match on it and prompt the user for another attempt.
        bail!("");
    };
    let settings = SettingsFile::new(settings_file());
    settings.with_mut(|s| {
        s.default_toolchain = Some(new_default.name.clone());
        Ok(())
    })?;
    info!("Default toolchain set to '{}'", new_default.name);
    Ok(())
}
