use std::cmp::Ordering::{Equal, Greater, Less};
use std::collections::HashMap;
use std::io::Write;
use std::str::FromStr;
use tracing::error;

use crate::{
    channel::Channel,
    commands::check::CheckCommand,
    component::SUPPORTED_PLUGINS,
    config::Config,
    fmt::{bold, colored_bold},
    target_triple::TargetTriple,
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

fn compare_and_print_versions(current_version: &Version, target_version: &Version) {
    match current_version.cmp(target_version) {
        Less => {
            colored_bold(Color::Yellow, |s| write!(s, "Update available"));
            println!(" : {} -> {}", current_version, target_version);
        }
        Equal => {
            colored_bold(Color::Green, |s| write!(s, "Up to date"));
            println!(" : {}", current_version);
        }
        Greater => {
            print!(" : {}", current_version);
            colored_bold(Color::Yellow, |s| write!(s, " (unstable)"));
            print!(" -> {}", target_version);
            colored_bold(Color::Green, |s| writeln!(s, " (recommended)"));
        }
    }
}

fn check_plugin(toolchain: &Toolchain, plugin: &str, latest_version: &Version) -> Result<()> {
    let plugin_executable = toolchain.path.join(&plugin);
    match std::process::Command::new(&plugin_executable)
        .arg("--version")
        .output()
    {
        Ok(o) => {
            let output = String::from_utf8_lossy(&o.stdout).into_owned();
            match output.split_whitespace().nth(1) {
                Some(v) => {
                    let version = Version::parse(v)?;
                    print!("    - ");
                    bold(|s| write!(s, "{}", plugin));
                    print!(" - ");
                    compare_and_print_versions(&version, latest_version);
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
        TargetTriple::from_component(component::FUELUP)?,
        None,
    ) {
        bold(|s| write!(s, "{} - ", component::FUELUP));
        compare_and_print_versions(
            &Version::parse(FUELUP_VERSION)?,
            &fuelup_download_cfg.version,
        );
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
                        DownloadCfg::new(
                            c,
                            TargetTriple::from_component(c)
                                .expect("Failed to create DownloadCfg from component"),
                            None,
                        )
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
                match output.split_whitespace().nth(1) {
                    Some(v) => {
                        let version = Version::parse(v)?;
                        bold(|s| write!(s, "  {} - ", &component));
                        compare_and_print_versions(&version, &latest_versions[component]);
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
                if plugin == &component::FORC_DEPLOY {
                    bold(|s| writeln!(s, "    - forc-client"));
                }
                if plugin == &component::FORC_RUN || plugin == &component::FORC_DEPLOY {
                    print!("  ");
                }
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
