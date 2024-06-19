use crate::{
    channel::Channel,
    commands::check::CheckCommand,
    config::Config,
    download::get_latest_version,
    file::get_bin_version,
    fmt::{bold, colored_bold},
    toolchain::{DistToolchainDescription, Toolchain},
};
use ansiterm::Color;
use anyhow::{anyhow, Result};
use component::{self, Components};
use semver::Version;
use std::{
    cmp::Ordering::{Equal, Greater, Less},
    collections::HashMap,
    path::Path,
    str::FromStr,
};
use tracing::{error, info};

fn collect_package_versions(channel: Channel) -> HashMap<String, Version> {
    let mut latest_versions: HashMap<String, Version> = HashMap::new();
    for (name, package) in channel.pkg {
        latest_versions.insert(name, package.version);
    }
    latest_versions
}

fn format_version_comparison(
    current_version: &Version,
    latest_version: &Version,
    verbose: bool,
) -> Option<String> {
    let order = current_version.cmp(latest_version);
    let s = match order {
        Less => {
            format!(
                "{} : {current_version} -> {latest_version}",
                colored_bold(Color::Yellow, "Update available")
            )
        }
        Greater => {
            format!(
                " : {} {} -> {} {}",
                current_version,
                colored_bold(Color::Yellow, "(unstable)"),
                latest_version,
                colored_bold(Color::Green, "(recommended)")
            )
        }
        Equal => {
            // Only show up-to-date message if verbose is true
            if verbose {
                format!(
                    "{} : {current_version}",
                    colored_bold(Color::Green, "Up to date")
                )
            } else {
                return None;
            }
        }
    };
    Some(s)
}

fn check_plugin(plugin_executable: &Path, plugin: &str, latest_version: &Version, verbose: bool) {
    if let Some(version_or_err) = match get_bin_version(plugin_executable) {
        Ok(version) => format_version_comparison(&version, latest_version, verbose),
        Err(err) => Some(err.to_string()),
    } {
        info!("{:>4}- {} - {}", "", plugin, version_or_err);
    }
}

fn check_fuelup(verbose: bool) -> Result<()> {
    let fuelup_version: Version = Version::parse(clap::crate_version!())?;
    if let Ok(latest) = get_latest_version(component::FUELUP) {
        if let Some(text) = format_version_comparison(&fuelup_version, &latest, verbose) {
            info!("{} - {}", bold(component::FUELUP), text);
        }
    } else {
        error!("Failed to get latest version for component 'fuelup'; skipping check for 'fuelup'");
    }
    Ok(())
}

fn check_toolchain(toolchain: &str, verbose: bool) -> Result<()> {
    let description = DistToolchainDescription::from_str(toolchain)?;
    let dist_channel = Channel::from_dist_channel(&description)?;
    let latest_package_versions = collect_package_versions(dist_channel);
    let toolchain = Toolchain::new(toolchain)?;
    info!("{}: {}", bold("Toolchain: "), &toolchain.name);

    let components = Components::collect_exclude_plugins()?;
    let plugins = component::Components::collect_plugins()?;
    components.iter().for_each(|component| {
        if let Some(latest_version) = latest_package_versions.get(&component.name) {
            let component_executable = toolchain.bin_path.join(&component.name);
            let version_text = match get_bin_version(&component_executable) {
                Ok(version) => format_version_comparison(&version, latest_version, verbose),
                Err(err) => Some(err.to_string()),
            };
            if let Some(version_text) = version_text {
                info!("{:>2}{} - {}", "", bold(&component.name), version_text);
            }
            if component.name == component::FORC {
                plugins.iter().for_each(|plugin| {
                    if !plugin.is_main_executable() {
                        info!("{:>4}- {}", "", bold(&plugin.name));
                    }
                    for (index, executable) in plugin.executables.iter().enumerate() {
                        let plugin_executable = toolchain.bin_path.join(executable);
                        let mut plugin_name = &plugin.name;
                        if !plugin.is_main_executable() {
                            print!("{:>2}", "");
                            plugin_name = plugin
                                .executables
                                .get(index)
                                .ok_or_else(|| anyhow!("Plugin name not found"))?;
                        }
                        let maybe_latest_version = plugin.publish.map_or_else(
                            || latest_package_versions.get(component::FORC),
                            |_| latest_package_versions.get(plugin_name),
                        );
                        if let Some(latest_version) = maybe_latest_version {
                            check_plugin(&plugin_executable, plugin_name, latest_version, verbose);
                        }
                    }
                });
            }
        }
    });
    Ok(())
}

pub fn check(command: CheckCommand) -> Result<()> {
    let CheckCommand { verbose } = command;
    let cfg = Config::from_env()?;
    for toolchain in cfg.list_dist_toolchains()? {
        check_toolchain(&toolchain, verbose)?;
    }
    check_fuelup(verbose)?;
    Ok(())
}
