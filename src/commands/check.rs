use anyhow::Result;
use clap::Parser;
use semver::Version;
use tracing::info;

use crate::{
    constants::{POSSIBLE_COMPONENTS, SUPPORTED_PLUGINS},
    download::DownloadCfg,
};

#[derive(Debug, Parser)]
pub struct CheckCommand {}

pub fn exec() -> Result<()> {
    for component in POSSIBLE_COMPONENTS {
        let mut latest_version: String = String::new();
        match std::process::Command::new(component)
            .arg("--version")
            .output()
        {
            Ok(o) => {
                let version = Version::parse(
                    String::from_utf8_lossy(&o.stdout)
                        .split_whitespace()
                        .collect::<Vec<&str>>()[1],
                )?;

                let download_cfg: DownloadCfg = DownloadCfg::new(component, None)?;
                if component == "forc" {
                    latest_version = download_cfg.version.to_string();
                }

                if version == download_cfg.version {
                    info!("{} - up to date: {}", component, version);
                } else {
                    info!(
                        "{} - update available: {} -> {}",
                        component, version, download_cfg.version
                    );
                }
            }
            Err(_) => info!("{} not found", component),
        };

        if component == "forc" {
            for plugin in SUPPORTED_PLUGINS {
                let plugin_component = component.to_owned() + "-" + plugin;
                match std::process::Command::new(&plugin_component)
                    .arg("--version")
                    .output()
                {
                    Ok(o) => {
                        let plugin_version = Version::parse(
                            String::from_utf8_lossy(&o.stdout)
                                .split_whitespace()
                                .collect::<Vec<&str>>()[1],
                        )?;

                        if plugin_version == Version::parse(&latest_version)? {
                            info!(" - {} - up to date: {}", plugin_component, latest_version);
                        } else {
                            info!(
                                " - {} - update available: {} -> {}",
                                plugin_component, plugin_version, latest_version
                            );
                        }
                    }
                    Err(_) => info!(" - {} not found", plugin_component),
                }
            }
        }
    }

    Ok(())
}
