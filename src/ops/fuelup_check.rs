use std::collections::HashMap;
use std::io::Write;

use crate::{
    channel::Channel,
    commands::check::CheckCommand,
    component::SUPPORTED_PLUGINS,
    config::Config,
    fmt::{bold, colored_bold},
    toolchain::Toolchain,
};
use anyhow::Result;
use semver::Version;
use termcolor::Color;

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
                    .nth(1)
                    .expect("expected version"),
            )?;
            print!("    - ");
            bold(|s| write!(s, "{}", plugin));
            print!(" - ");
            if &version == latest_version {
                colored_bold(Color::Green, |s| write!(s, "Up to date"));
                println!(" : {}", version);
            } else {
                colored_bold(Color::Yellow, |s| write!(s, "Update available"));
                println!(" : {} -> {}", version, latest_version);
            }
        }
        Err(e) => {
            print!("    - ");
            bold(|s| write!(s, "{}", plugin));
            print!(" - ");
            if plugin_executable.exists() {
                println!("execution error - {}", e);
            } else {
                println!("not found");
            }
        }
    }
    Ok(())
}

fn collect_versions(channel: Channel) -> Result<HashMap<String, Version>> {
    let mut latest_versions: HashMap<String, Version> = HashMap::new();
    for package in channel.packages {
        latest_versions.insert(package.name.to_string(), package.version.clone());
    }
    let fuelup_download_cfg: DownloadCfg = DownloadCfg::new(component::FUELUP, None)?;
    latest_versions.insert(fuelup_download_cfg.name, fuelup_download_cfg.version);
    Ok(latest_versions)
}

pub fn check(command: CheckCommand) -> Result<()> {
    let CheckCommand { verbose } = command;

    let cfg = Config::from_env()?;
    let toolchains = cfg.list_toolchains()?;
    let channel = Channel::from_dist_channel("latest")?;
    let latest_versions = collect_versions(channel)?;

    for toolchain in toolchains {
        let toolchain = Toolchain::from(&toolchain);
        bold(|s| writeln!(s, "{}", &toolchain.name));
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
                            .nth(1)
                            .expect("expected version"),
                    )?;

                    bold(|s| write!(s, "  {} - ", &component));
                    if version == latest_versions[component] {
                        colored_bold(Color::Green, |s| write!(s, "Up to date"));
                        println!(" : {}", version);
                    } else {
                        colored_bold(Color::Yellow, |s| write!(s, "Update available"));
                        println!(" : {} -> {}", version, latest_versions[component]);
                    }
                }
                Err(e) => {
                    print!("  ");
                    bold(|s| write!(s, "{}", &component));
                    print!(" - ");
                    if component_executable.exists() {
                        println!("execution error - {}", e);
                    } else {
                        println!("not found");
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

    const FUELUP_VERSION: &str = clap::crate_version!();

    bold(|s| write!(s, "{} - ", component::FUELUP));
    if FUELUP_VERSION == latest_versions[component::FUELUP].to_string() {
        colored_bold(Color::Green, |s| write!(s, "Up to date"));
        println!(" : {}", FUELUP_VERSION);
    } else {
        colored_bold(Color::Yellow, |s| write!(s, "Update available"));
        println!(
            " : {} -> {}",
            FUELUP_VERSION,
            latest_versions[component::FUELUP]
        );
    };
    Ok(())
}
