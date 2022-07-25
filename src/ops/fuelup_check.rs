use std::collections::HashMap;
use std::io::Write;
use std::str::FromStr;
use tracing::error;

use crate::{
    channel::Channel,
    commands::check::CheckCommand,
    component::SUPPORTED_PLUGINS,
    config::Config,
    download::target_from_name,
    fmt::{bold, colored_bold},
    toolchain::{DistToolchainName, Toolchain},
};
use anyhow::Result;
use semver::Version;
use termcolor::Color;

use crate::{component, download::DownloadCfg};

fn collect_versions(channel: Channel) -> HashMap<String, Version> {
    let mut latest_versions: HashMap<String, Version> = HashMap::new();
    for (name, package) in channel.pkg.into_iter() {
        latest_versions.insert(name, package.version);
    }

    latest_versions
}

fn check_plugin(toolchain: &Toolchain, plugin: &str, latest_version: &Version) -> Result<()> {
    let plugin_executable = toolchain.path.join(&plugin);
    match std::process::Command::new(&plugin_executable)
        .arg("--version")
        .output()
    {
        Ok(o) => {
            let output = String::from_utf8_lossy(&o.stdout).into_owned();
            match output.split_whitespace().rev().next() {
                Some(v) => {
                    let version = Version::parse(v)?;
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
                None => {
                    eprintln!("    - {} - Error getting version string", plugin);
                }
            };
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

fn check_fuelup() -> Result<()> {
    const FUELUP_VERSION: &str = clap::crate_version!();

    if let Ok(fuelup_download_cfg) = DownloadCfg::new(
        component::FUELUP,
        target_from_name(component::FUELUP).ok(),
        None,
    ) {
        bold(|s| write!(s, "{} - ", component::FUELUP));
        if FUELUP_VERSION == fuelup_download_cfg.version.to_string() {
            colored_bold(Color::Green, |s| write!(s, "Up to date"));
            println!(" : {}", FUELUP_VERSION);
        } else {
            colored_bold(Color::Yellow, |s| write!(s, "Update available"));
            println!(" : {} -> {}", FUELUP_VERSION, fuelup_download_cfg.version);
        };
    } else {
        error!("Failed to create DownloadCfg for component 'fuelup'; skipping check for 'fuelup'");
    }
    Ok(())
}

fn check_toolchain(toolchain: &str, verbose: bool) -> Result<()> {
    let latest_versions = match Channel::from_dist_channel(&DistToolchainName::from_str(toolchain)?)
    {
        Ok(c) => collect_versions(c),
        Err(e) => {
            error!(
                "Failed to get latest channel {} - fetching versions using GitHub API",
                e
            );
            [component::FORC, component::FUEL_CORE, component::FUELUP]
                .iter()
                .map(|&c| {
                    (
                        c.to_owned(),
                        DownloadCfg::new(c, target_from_name(c).ok(), None)
                            .unwrap()
                            .version,
                    )
                })
                .collect()
        }
    };

    let toolchain = Toolchain::from(toolchain)?;
    bold(|s| writeln!(s, "{}", &toolchain.name));
    for component in [component::FORC, component::FUEL_CORE] {
        let component_executable = toolchain.path.join(component);

        match std::process::Command::new(&component_executable)
            .arg("--version")
            .output()
        {
            Ok(o) => {
                let output = String::from_utf8_lossy(&o.stdout).into_owned();
                match output.split_whitespace().rev().next() {
                    Some(v) => {
                        let version = Version::parse(v)?;

                        bold(|s| write!(s, "  {} - ", &component));
                        if version == latest_versions[component] {
                            colored_bold(Color::Green, |s| write!(s, "Up to date"));
                            println!(" : {}", version);
                        } else {
                            colored_bold(Color::Yellow, |s| write!(s, "Update available"));
                            println!(" : {} -> {}", version, latest_versions[component]);
                        }
                    }
                    None => {
                        eprintln!("  {} - Error getting version string", component);
                    }
                };
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
    Ok(())
}

pub fn check(command: CheckCommand) -> Result<()> {
    let CheckCommand { verbose } = command;

    let cfg = Config::from_env()?;

    for toolchain in cfg.list_official_toolchains()? {
        check_toolchain(&toolchain, verbose)?;
    }

    check_fuelup()?;
    Ok(())
}
