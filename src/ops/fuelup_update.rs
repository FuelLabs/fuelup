use crate::{
    channel::Channel,
    config::Config,
    fmt::{bold, colored_bold},
    path::warn_existing_fuel_executables,
    toolchain::{DistToolchainDescription, Toolchain},
};
use anyhow::{bail, Result};
use std::io::Write;
use std::str::FromStr;
use termcolor::Color;
use tracing::info;

const UPDATED: &str = "updated";
const PARTIALLY_UPDATED: &str = "partially updated";
const UNCHANGED: &str = "unchanged";

pub fn update() -> Result<()> {
    let config = Config::from_env()?;
    let toolchains = config.list_dist_toolchains()?;
    let mut summary: Vec<(String, String)> = Vec::with_capacity(toolchains.len());

    warn_existing_fuel_executables()?;

    for toolchain in toolchains {
        let mut installed_bins = String::new();
        let mut errored_bins = String::new();

        let description = DistToolchainDescription::from_str(&toolchain)?;
        info!("updating the '{}' toolchain", description);

        let (cfgs, hash) = if let Ok((channel, hash)) = Channel::from_dist_channel(&description) {
            if let Ok(true) = config.hash_matches(&description, &hash) {
                info!("'{}' already installed and up to date", description);
                summary.push((format!("{} {}", toolchain, UNCHANGED), "".to_string()));
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
                Ok(cfg) => installed_bins.push_str(&format!("  - {} {}\n", cfg.name, cfg.version)),
                Err(e) => errored_bins.push_str(&format!("  - {}\n", e)),
            };
        }

        let mut status = String::new();
        if !installed_bins.is_empty() {
            status = UPDATED.to_string();
            installed_bins = format!("  updated components:\n{}", installed_bins);
        }

        if errored_bins.is_empty() {
            config.save_hash(&toolchain, &hash)?;
        } else {
            status = PARTIALLY_UPDATED.to_string();
            errored_bins = format!("  failed to update:\n{}", errored_bins);
        };

        summary.push((
            format!("{} {}\n", toolchain, status),
            format!("{}{}", installed_bins, errored_bins),
        ));
    }

    info!("");
    for (toolchain_info, components_info) in summary {
        if !toolchain_info
            .matches(UPDATED)
            .collect::<String>()
            .is_empty()
        {
            colored_bold(Color::Green, |s| write!(s, "{}", toolchain_info));
        } else {
            bold(|s| write!(s, "{}", toolchain_info));
        }
        info!("{}", components_info);
    }

    Ok(())
}
