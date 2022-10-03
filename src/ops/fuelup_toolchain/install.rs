use crate::download::DownloadCfg;
use crate::path::{fuelup_dir, settings_file};
use crate::settings::SettingsFile;
use crate::toolchain::{OfficialToolchainDescription, Toolchain};
use crate::{channel::Channel, commands::toolchain::InstallCommand};
use anyhow::{bail, Result};
use std::fmt::Write;
use std::str::FromStr;
use tempfile::tempdir_in;
use tracing::{error, info};

pub fn install(command: InstallCommand) -> Result<()> {
    let InstallCommand { name } = command;

    let toolchain = Toolchain::new(&name)?;
    let description = OfficialToolchainDescription::from_str(&name)?;

    let settings = SettingsFile::new(settings_file());
    settings.with_mut(|s| {
        s.default_toolchain = Some(toolchain.name.clone());
        Ok(())
    })?;

    let mut errored_bins = String::new();
    let mut installed_bins = String::new();

    let fuelup_dir = fuelup_dir();
    let tmp_dir = tempdir_in(&fuelup_dir)?;
    let tmp_dir_path = tmp_dir.into_path();

    let cfgs: Vec<DownloadCfg> =
        if let Ok(channel) = Channel::from_dist_channel(&description, tmp_dir_path) {
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
            Err(e) => writeln!(errored_bins, "- {}", e)?,
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
