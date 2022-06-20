use std::collections::HashMap;

use crate::{
    colors::{print_bold, print_boldln, print_with_color},
    commands::check::CheckCommand,
    component::SUPPORTED_PLUGINS,
    config::Config,
    toolchain::Toolchain,
};
use anyhow::Result;
use semver::Version;
use termcolor::Color;
use tracing::info;

use crate::{component, download::DownloadCfg};

fn check_plugin(toolchain: &Toolchain, plugin: &str, latest_version: &Version) -> Result<()> {
    let plugin_executable = toolchain.path.join(&plugin);
    match std::process::Command::new(&plugin_executable)
        .arg("--version")
        .output()
    {
        Ok(o) => {
            let version = Version::parse(
                String::from_utf8_lossy(&o.stdout)
                    .split_whitespace()
                    .collect::<Vec<&str>>()[1],
            )?;

            if &version == latest_version {
                print!("    - {} - ", plugin);
                print_with_color("Up to date ", Color::Green).expect("Internal printing error");
                println!(": {}", version);
            } else {
                print!("    - {} - ", plugin);
                print_with_color("Update available ", Color::Yellow)
                    .expect("Internal printing error");
                println!("{} -> {}", version, latest_version);
            }
        }
        Err(e) => {
            print!("    - {} : ", plugin);
            if plugin_executable.exists() {
                info!("execution error - {}", e);
            } else {
                info!("not found");
            }
        }
    }
    Ok(())
}

pub fn check(command: CheckCommand) -> Result<()> {
    let CheckCommand { verbose } = command;

    let cfg = Config::from_env()?;
    let toolchains = cfg.list_toolchains()?;
    let mut latest_versions: HashMap<String, Version> = HashMap::new();
    let components = Vec::from([component::FORC, component::FUEL_CORE, component::FUELUP]);

    for component in components {
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

                if verbose && component == component::FORC {
                    for plugin in SUPPORTED_PLUGINS {
                        check_plugin(&toolchain, plugin, &latest_versions[component::FORC])?;
                    }
                }
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
        Err(e) => {
            // Unclear how we might run into this if we run it from fuelup - print errors anyway
            print_bold(&format!("  {} : ", component::FUELUP))?;
            info!("execution error - {}", e);
        }
    };
    Ok(())
}
