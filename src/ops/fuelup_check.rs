use std::collections::HashMap;

use crate::{
    colors::{print_bold, print_boldln, print_with_color},
    config::Config,
    toolchain::Toolchain,
};
use anyhow::Result;
use clap::Parser;
use semver::Version;
use termcolor::Color;
use tracing::info;

use crate::{component, download::DownloadCfg};

#[derive(Debug, Parser)]
pub struct CheckCommand {}

pub fn check() -> Result<()> {
    let cfg = Config::from_env()?;
    let toolchains = cfg.list_toolchains()?;
    let mut latest_versions: HashMap<String, Version> = HashMap::new();

    for component in [component::FORC, component::FUEL_CORE, component::FUELUP] {
        let download_cfg: DownloadCfg = DownloadCfg::new(component, None)?;
        latest_versions.insert(component.to_string(), download_cfg.version);
    }

    for toolchain in toolchains {
        if let Ok(toolchain) = Toolchain::from(&toolchain) {
            print_boldln(&toolchain.name)?;
            for component in [component::FORC, component::FUEL_CORE] {
                let component_executable = toolchain.path.join(component);

                match std::process::Command::new(&component_executable)
                    .arg("--version")
                    .output()
                {
                    Ok(o) => {
                        let version = Version::parse(
                            String::from_utf8_lossy(&o.stdout)
                                .split_whitespace()
                                .collect::<Vec<&str>>()[1],
                        )?;

                        print_bold(&format!("  {} - ", component))?;
                        if version == latest_versions[component] {
                            print_with_color("Up to date ", Color::Green)?;
                            println!(": {}", version);
                        } else {
                            print_with_color("Update available : ", Color::Yellow)?;
                            println!("{} -> {}", version, latest_versions[component]);
                        }
                    }
                    Err(e) => {
                        print_bold(&format!("  {} : ", component))?;
                        if component_executable.exists() {
                            info!("execution error - {}", e);
                        } else {
                            info!("not found");
                        }
                    }
                };
            }
        }
    }

    match std::process::Command::new(component::FUELUP)
        .arg("--version")
        .output()
    {
        Ok(o) => {
            let version = Version::parse(
                String::from_utf8_lossy(&o.stdout)
                    .split_whitespace()
                    .collect::<Vec<&str>>()[1],
            )?;

            print_bold(&format!("{} - ", component::FUELUP))?;
            if version == latest_versions[component::FUELUP] {
                print_with_color("Up to date ", Color::Green)?;
                println!(": {}", version);
            } else {
                print_with_color("Update available : ", Color::Yellow)?;
                println!("{} -> {}", version, latest_versions[component::FUELUP]);
            }
        }
        Err(_) => {
            print_bold(&format!("  {}", component::FUELUP))?;
            info!(" not found");
        }
    };
    Ok(())
}
