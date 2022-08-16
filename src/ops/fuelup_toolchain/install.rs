use crate::component;
use crate::download::DownloadCfg;
use crate::path::settings_file;
use crate::settings::SettingsFile;
use crate::toolchain::{DistToolchainName, TargetTriple, Toolchain};
use crate::{channel::Channel, commands::toolchain::InstallCommand};
use anyhow::Result;
use std::fmt::Write;
use tracing::{error, info};

pub fn install(command: InstallCommand) -> Result<()> {
    let InstallCommand { name } = command;

    let toolchain = Toolchain::new(&name, None)?;

    let settings = SettingsFile::new(settings_file());
    settings.with_mut(|s| {
        s.default_toolchain = Some(toolchain.name.clone());
        Ok(())
    })?;

    let mut errored_bins = String::new();
    let mut installed_bins = String::new();

    let cfgs: Vec<DownloadCfg> = match Channel::from_dist_channel(&DistToolchainName::Latest) {
        Ok(c) => c.build_download_configs(),
        Err(e) => {
            error!(
                "Failed to get latest channel {} - fetching versions using GitHub API",
                e
            );
            [component::FORC, component::FUEL_CORE, component::FORC_LSP]
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
