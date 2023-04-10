use crate::path::{settings_file, warn_existing_fuel_executables};
use crate::settings::SettingsFile;
use crate::toolchain::{DistToolchainDescription, Toolchain};
use crate::{channel::Channel, commands::toolchain::InstallCommand};
use anyhow::{bail, Result};
use std::fmt::Write;
use std::str::FromStr;
use tracing::{error, info};

pub fn install(command: InstallCommand) -> Result<()> {
    let InstallCommand { name } = command;

    let description = DistToolchainDescription::from_str(&name)?;

    let settings_file = settings_file();
    if !settings_file.exists() {
        let settings = SettingsFile::new(settings_file);
        settings.with_mut(|s| {
            s.default_toolchain = Some(description.to_string());
            Ok(())
        })?;
    }

    let mut errored_bins = String::new();
    let mut installed_bins = String::new();

    warn_existing_fuel_executables()?;

    let toolchain = Toolchain::from_path(&description.to_string());
    let cfgs = if let Ok(channel) = Channel::from_dist_channel(&description) {
        channel.build_download_configs()
    } else {
        bail!("Could not build download configs from channel")
    };

    info!(
        "Downloading: {}",
        cfgs.iter()
            .map(|c| c.name.clone() + " ")
            .collect::<String>()
    );

    for cfg in cfgs {
        match toolchain.add_component(cfg) {
            Ok(cfg) => writeln!(installed_bins, "- {} {}", cfg.name, cfg.version)?,
            Err(e) => writeln!(errored_bins, "- {e}")?,
        };
    }

    if errored_bins.is_empty() {
        info!("\nInstalled:\n{}", installed_bins);
        info!("\nThe Fuel toolchain is installed and up to date");
    } else if installed_bins.is_empty() {
        error!("\nfuelup failed to install:\n{}", errored_bins)
    } else {
        info!(
            "\nThe Fuel toolchain is partially installed.\nfuelup failed to install: {}",
            errored_bins
        );
    };

    Ok(())
}
