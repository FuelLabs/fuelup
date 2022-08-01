use std::str::FromStr;

use anyhow::{bail, Result};
use semver::Version;

use crate::{
    commands::component::AddCommand,
    download::{target_from_name, DownloadCfg},
    path::settings_file,
    settings::SettingsFile,
    toolchain::{DistToolchainName, Toolchain},
};

pub fn add(command: AddCommand) -> Result<()> {
    let AddCommand {
        maybe_versioned_component,
    } = command;

    let settings = SettingsFile::new(settings_file());
    let current_default = match settings.with(|s| Ok(s.default_toolchain.clone()))? {
        Some(t) => t,
        None => {
            bail!("No default toolchain detected. Please install or create a toolchain first.")
        }
    };

    let toolchain = Toolchain::from(&current_default)?;
    if let Some((component, version)) = maybe_versioned_component.split_once('@') {
        if let Ok(toolchain) = DistToolchainName::from_str(&current_default) {
            bail!(
                "You cannot specify versions of components to add to official toolchain '{}'",
                toolchain
            );
        }

        println!("installing component {} to {}", component, &current_default);
        let download_cfg = DownloadCfg::new(
            component,
            Some(target_from_name(component)?),
            Some(Version::from_str(version)?),
        )?;
        toolchain.add_component(download_cfg)?;
    } else {
        println!(
            "installing component {} to {}",
            maybe_versioned_component, &current_default
        );
        let download_cfg = DownloadCfg::new(
            &maybe_versioned_component,
            Some(target_from_name(&maybe_versioned_component)?),
            None,
        )?;
        toolchain.add_component(download_cfg)?;
    }

    Ok(())
}
