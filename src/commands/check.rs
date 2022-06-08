use anyhow::Result;
use clap::Parser;
use tracing::info;

use crate::{
    constants::{POSSIBLE_COMPONENTS, SUPPORTED_PLUGINS},
    download::DownloadCfg,
};

#[derive(Debug, Parser)]
pub struct CheckCommand {}

pub fn exec() -> Result<()> {
    for component in POSSIBLE_COMPONENTS.iter() {
        let mut latest_version = String::new();
        match std::process::Command::new(component)
            .arg("--version")
            .output()
        {
            Ok(o) => {
                let version = "v".to_owned()
                    + String::from_utf8_lossy(&o.stdout)
                        .split_whitespace()
                        .collect::<Vec<&str>>()[1];

                let download_cfg: DownloadCfg =
                    DownloadCfg::new(component, None).unwrap_or_else(|_| {
                        panic!("Could not create download config for {}", component)
                    });
                if component == &"forc" {
                    latest_version = download_cfg.version.clone();
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

        if component == &"forc" {
            for plugin in SUPPORTED_PLUGINS {
                match std::process::Command::new(component)
                    .arg(plugin)
                    .arg("--version")
                    .output()
                {
                    Ok(o) => {
                        let plugin_version = "v".to_owned()
                            + String::from_utf8_lossy(&o.stdout)
                                .split_whitespace()
                                .collect::<Vec<&str>>()[1];

                        if plugin_version == latest_version {
                            info!(
                                " - {}-{} - up to date: {}",
                                component, plugin, latest_version
                            );
                        } else {
                            info!(
                                " - {}-{} - update available: {} -> {}",
                                component, plugin, plugin_version, latest_version
                            );
                        }
                    }
                    Err(_) => info!(" - {} not found", component),
                }
            }
        }
    }

    Ok(())
}
