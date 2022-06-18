use crate::colors::print_with_color;
use anyhow::Result;
use clap::Parser;
use semver::Version;
use termcolor::Color;
use tracing::info;

use crate::{component, download::DownloadCfg};

#[derive(Debug, Parser)]
pub struct CheckCommand {}

pub mod plugin {
    pub const FMT: &str = "fmt";
    pub const LSP: &str = "lsp";
    pub const EXPLORE: &str = "explore";
}

pub fn check() -> Result<()> {
    for component in [component::FORC, component::FUEL_CORE, component::FUELUP] {
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
                if component == component::FORC {
                    latest_version = download_cfg.version.to_string();
                }

                if version == download_cfg.version {
                    print!("{} - ", component);
                    print_with_color("Up to date ", Color::Green);
                    println!(": {}", version);
                } else {
                    print!("{} - ", component);
                    print_with_color("Update available ", Color::Yellow);
                    println!("{} -> {}", version, download_cfg.version);
                }
            }
            Err(_) => info!("{} not found", component),
        };

        if component == component::FORC {
            for plugin in [plugin::FMT, plugin::LSP, plugin::EXPLORE] {
                let plugin_component = component.to_owned() + "-" + plugin;
                match std::process::Command::new(&plugin_component)
                    .arg("--version")
                    .output()
                {
                    Ok(o) => {
                        let version = Version::parse(
                            String::from_utf8_lossy(&o.stdout)
                                .split_whitespace()
                                .collect::<Vec<&str>>()[1],
                        )?;

                        if version == Version::parse(&latest_version)? {
                            print!(" - {} - ", plugin_component);
                            print_with_color("Up to date ", Color::Green);
                            println!(": {}", version);
                        } else {
                            print!(" - {} - ", plugin_component);
                            print_with_color("Update available ", Color::Yellow);
                            println!("{} -> {}", version, latest_version);
                        }
                    }
                    Err(_) => info!(" - {} not found", plugin_component),
                }
            }
        }
    }

    Ok(())
}
