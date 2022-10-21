use crate::{
    channel::Channel,
    config::Config,
    toolchain::{OfficialToolchainDescription, Toolchain},
};
use anyhow::{bail, Result};
use std::fmt::Write;
use std::str::FromStr;
use tracing::{error, info};

pub fn update() -> Result<()> {
    let config = Config::from_env()?;

    for toolchain in config.list_official_toolchains()? {
        let mut errored_bins = String::new();
        let mut installed_bins = String::new();

        let description = OfficialToolchainDescription::from_str(&toolchain)?;

        let (cfgs, hash) = if let Ok((channel, hash)) = Channel::from_dist_channel(&description) {
            if config.hash_matches(&description, &hash) {
                info!("'{}' is already installed and up to date", toolchain);
                continue;
            };
            (channel.build_download_configs(), hash)
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
            let toolchain = Toolchain::from_path(&description.to_string())?;
            match toolchain.add_component(cfg) {
                Ok(cfg) => writeln!(installed_bins, "- {} {}", cfg.name, cfg.version)?,
                Err(e) => writeln!(errored_bins, "- {}", e)?,
            };
        }

        if errored_bins.is_empty() {
            config.save_hash(&toolchain, &hash)?;
            info!("\nUpdated:\n{}", installed_bins);
            info!("\nThe Fuel toolchain is installed and up to date");
        } else if installed_bins.is_empty() {
            error!("\nfuelup failed to install:\n{}", errored_bins)
        } else {
            info!(
                "\nThe Fuel toolchain is partially installed.\nfuelup failed to install: {}",
                errored_bins
            );
        };
    }

    Ok(())
}
