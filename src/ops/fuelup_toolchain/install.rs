use crate::component;
use crate::constants::{CHANNEL_LATEST_FILE_NAME, CHANNEL_NIGHTLY_FILE_NAME};
use crate::download::DownloadCfg;
use crate::path::{fuelup_dir, settings_file};
use crate::settings::SettingsFile;
use crate::target_triple::TargetTriple;
use crate::toolchain::{DistToolchainName, OfficialToolchainDescription, Toolchain};
use crate::{channel::Channel, commands::toolchain::InstallCommand};
use anyhow::Result;
use std::fmt::Write;
use std::fs;
use std::str::FromStr;
use tempfile::tempdir_in;
use tracing::{error, info};

pub fn install(command: InstallCommand) -> Result<()> {
    let InstallCommand { name } = command;

    let description = OfficialToolchainDescription::from_str(&name)?;
    println!("{}", description);

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

    let fuelup_dir = fuelup_dir();
    let tmp_dir = tempdir_in(&fuelup_dir)?;
    let tmp_dir_path = tmp_dir.into_path();

    let cfgs: Vec<DownloadCfg> =
        match Channel::from_dist_channel(&description, tmp_dir_path.clone()) {
            Ok(c) => c.build_download_configs(),
            Err(e) => {
                error!(
                    "Failed to get latest channel {} - fetching versions using GitHub API",
                    e
                );
                [component::FORC, component::FUEL_CORE]
                    .iter()
                    .map(|c| {
                        DownloadCfg::new(
                            c,
                            TargetTriple::from_component(c)
                                .expect("Failed to create DownloadCfg from component"),
                            None,
                        )
                        .unwrap()
                    })
                    .collect()
            }
        };

    info!(
        "Downloading: {}",
        cfgs.iter()
            .map(|c| c.name.clone() + " ")
            .collect::<String>()
    );

    let toolchain = Toolchain::from_path(&description.to_string())?;
    for cfg in cfgs {
        match toolchain.add_component(cfg) {
            Ok(cfg) => writeln!(installed_bins, "- {} {}", cfg.name, cfg.version)?,
            Err(e) => writeln!(errored_bins, "- {}", e)?,
        };
    }

    let channel_file_name = match description.name {
        DistToolchainName::Latest => CHANNEL_LATEST_FILE_NAME,
        DistToolchainName::Nightly => CHANNEL_NIGHTLY_FILE_NAME,
    };
    if errored_bins.is_empty() {
        fs::copy(
            tmp_dir_path.join(channel_file_name),
            toolchain.path.join(channel_file_name),
        )?;
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
