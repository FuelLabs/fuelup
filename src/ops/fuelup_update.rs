use crate::{
    channel::Channel,
    config::Config,
    fmt::{bold, colored_bold},
    path::warn_existing_fuel_executables,
    toolchain::{DistToolchainDescription, Toolchain},
};
use ansiterm::Color;
use anyhow::{bail, Result};
use std::str::FromStr;
use tracing::{error, info};

const UPDATED: &str = "updated";
const PARTIALLY_UPDATED: &str = "partially updated";

pub fn update() -> Result<()> {
    let config = Config::from_env()?;
    let toolchains = config.list_dist_toolchains()?;
    let mut summary: Vec<(String, String)> = Vec::with_capacity(toolchains.len());

    warn_existing_fuel_executables()?;

    if toolchains.is_empty() {
        error!(
            "Could find any toolchain. Please run `fuelup default <toolchain>` to install toolchains first."
        );
        return Ok(());
    }

    for toolchain in toolchains {
        let mut installed_bins = String::new();
        let mut errored_bins = String::new();

        let description = DistToolchainDescription::from_str(&toolchain)?;
        info!("updating the '{}' toolchain", description);

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
            let toolchain = Toolchain::from_path(&description.to_string());
            match toolchain.add_component(cfg) {
                Ok(cfg) => installed_bins.push_str(&format!("  - {} {}\n", cfg.name, cfg.version)),
                Err(e) => errored_bins.push_str(&format!("  - {e}\n")),
            };
        }

        let mut status = String::new();
        if !installed_bins.is_empty() {
            status = UPDATED.to_string();
            installed_bins = format!("{:>2}updated components:\n{}", "", installed_bins);
        }

        if !errored_bins.is_empty() {
            status = PARTIALLY_UPDATED.to_string();
            errored_bins = format!("{:>2}failed to update:\n{}", "", errored_bins);
        };

        summary.push((
            format!("{toolchain} {status}"),
            format!("{installed_bins}{errored_bins}"),
        ));
    }

    info!("");
    for (toolchain_info, components_info) in summary {
        if !toolchain_info
            .matches(UPDATED)
            .collect::<String>()
            .is_empty()
        {
            info!("{}", colored_bold(Color::Green, &toolchain_info));
        } else {
            info!("{}", bold(&toolchain_info));
        }
        info!("{}", components_info);
    }

    Ok(())
}
