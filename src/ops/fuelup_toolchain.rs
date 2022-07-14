use crate::download::DownloadCfg;
use crate::path::settings_file;
use crate::settings::SettingsFile;
use crate::toolchain::Toolchain;
use crate::{channel::Channel, commands::toolchain::InstallCommand};
use anyhow::Result;
use std::fmt::Write;
use tracing::{error, info};

pub mod toolchain {
    pub const LATEST: &str = "latest";
}

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
    let mut download_msg = String::new();

    let mut cfgs: Vec<DownloadCfg> = Vec::new();

    let channel = Channel::from_dist_channel("latest")?;

    for package in channel.packages.iter() {
        write!(download_msg, "{} ", package.name)?;
        let download_cfg: DownloadCfg =
            DownloadCfg::new(&package.name, Some(package.version.clone()))?;
        cfgs.push(download_cfg);
    }

    info!("Downloading: {}", download_msg);
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
