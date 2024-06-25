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
use anyhow::Result;
use component::{self, Components};
use semver::Version;
use std::{
    cmp::Ordering::{self, Equal, Greater, Less},
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
    order: &Ordering,
    current_version: &Version,
    latest_version: &Version,
) -> String {
    match order {
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
            format!(
                "{} : {current_version}",
                colored_bold(Color::Green, "Up to date")
            )
        }
    }
}

fn check_plugin(
    plugin_executable: &Path,
    plugin: &str,
    latest_version: &Version,
    verbose: bool,
    num_updates: &mut u16,
) {
    let version_or_err = match get_bin_version(plugin_executable) {
        Ok(version) => {
            let order = version.cmp(latest_version);
            if order != Equal {
                *num_updates += 1;
            }
            format_version_comparison(&order, &version, latest_version)
        }
        Err(err) => err.to_string(),
    };
    if verbose {
        info!("{:>4}- {} - {}", "", plugin, version_or_err);
    }
}

fn check_fuelup(verbose: bool, max_length: usize) -> Result<()> {
    let fuelup_version: Version = Version::parse(clap::crate_version!())?;
    if let Ok(latest) = get_latest_version(component::FUELUP) {
        let order = fuelup_version.cmp(&latest);
        if verbose {
            let res = format_version_comparison(&order, &fuelup_version, &latest);
            info!("{} - {}", bold(component::FUELUP), res);
        } else {
            let text = if order == Equal {
                colored_bold(Color::Green, "Up to date")
            } else {
                colored_bold(Color::Yellow, "Update available")
            };
            let padded_fuelup = format!("{:<width$}", "fuelup", width = max_length);
            info!("{} - {}", padded_fuelup, text);
        }
    } else {
        error!("Failed to get latest version for component 'fuelup'; skipping check for 'fuelup'");
    }
    Ok(())
}

fn check_toolchain(toolchain: &str, verbose: bool) -> Result<u16> {
    let description = DistToolchainDescription::from_str(toolchain)?;
    let dist_channel = Channel::from_dist_channel(&description)?;
    let latest_package_versions = collect_package_versions(dist_channel);
    let toolchain = Toolchain::new(toolchain)?;
    if verbose {
        info!("{}: {}", bold("Toolchain: "), &toolchain.name);
    }

    let components = Components::collect_exclude_plugins()?;
    let plugins = component::Components::collect_plugins()?;
    let mut num_updates = 0;
    components.iter().for_each(|component| {
        if let Some(latest_version) = latest_package_versions.get(&component.name) {
            let component_executable = toolchain.bin_path.join(&component.name);
            let version_text = match get_bin_version(&component_executable) {
                Ok(version) => {
                    let order = version.cmp(latest_version);
                    format_version_comparison(&order, &version, latest_version)
                }
                Err(err) => err.to_string(),
            };
            if verbose {
                info!("{:>2}{} - {}", "", bold(&component.name), version_text);
            }
            if component.name == component::FORC {
                plugins.iter().for_each(|plugin| {
                    if !plugin.is_main_executable() && verbose {
                        info!("{:>4}- {}", "", bold(&plugin.name));
                    }
                    for (index, executable) in plugin.executables.iter().enumerate() {
                        let plugin_executable = toolchain.bin_path.join(executable);
                        let mut plugin_name = &plugin.name;
                        if !plugin.is_main_executable() && verbose {
                            print!("{:>2}", "");
                            if let Some(exe_name) = plugin.executables.get(index) {
                                plugin_name = exe_name;
                            } else {
                                error!("Plugin name not found");
                            }
                        }
                        let maybe_latest_version = plugin.publish.map_or_else(
                            || latest_package_versions.get(component::FORC),
                            |_| latest_package_versions.get(plugin_name),
                        );
                        if let Some(latest_version) = maybe_latest_version {
                            check_plugin(
                                &plugin_executable,
                                plugin_name,
                                latest_version,
                                verbose,
                                &mut num_updates,
                            );
                        }
                    }
                });
            }
        }
    });
    Ok(num_updates)
}

pub fn check(command: CheckCommand) -> Result<()> {
    let CheckCommand { verbose } = command;
    let cfg = Config::from_env()?;

    // Find the maximum length of toolchain names
    let toolchains = cfg.list_dist_toolchains()?;
    let max_length = toolchains.iter().map(|t| t.len()).max().unwrap_or(0);

    for toolchain in toolchains {
        let num_updates = check_toolchain(&toolchain, verbose)?;
        if !verbose {
            let s = if num_updates == 0 {
                colored_bold(Color::Green, "Up to date")
            } else {
                colored_bold(
                    Color::Yellow,
                    &format!("Update available ({num_updates} updates)",),
                )
            };
            // Pad the toolchain name with spaces to align the `-` signs
            let padded_toolchain = format!("{:<width$}", toolchain, width = max_length);
            info!("{} - {}", &padded_toolchain, s);
        }
    }
    check_fuelup(verbose, max_length)?;
    Ok(())
}
