use crate::{
    channel::Channel,
    commands::check::CheckCommand,
    config::Config,
    download::DownloadCfg,
    file::get_bin_version,
    fmt::{bold, colored_bold},
    target_triple::TargetTriple,
    toolchain::{DistToolchainDescription, Toolchain},
};
use ansiterm::Color;
use anyhow::Result;
use component::{self, Components};
use semver::Version;
use std::collections::HashMap;
use std::str::FromStr;
use std::{
    cmp::Ordering::{Equal, Greater, Less},
    path::Path,
};
use tracing::{error, info};

fn collect_package_versions(channel: Channel) -> HashMap<String, Version> {
    let mut latest_versions: HashMap<String, Version> = HashMap::new();

    for (name, package) in channel.pkg.into_iter() {
        latest_versions.insert(name, package.version);
    }

    latest_versions
}

fn format_version_comparison(current_version: &Version, latest_version: &Version) -> String {
    match current_version.cmp(latest_version) {
        Less => {
            format!(
                "{} : {current_version} -> {latest_version}",
                colored_bold(Color::Yellow, "Update available")
            )
        }
        Equal => {
            format!(
                "{} : {current_version}",
                colored_bold(Color::Green, "Up to date")
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
    }
}

fn check_plugin(plugin_executable: &Path, plugin: &str, latest_version: &Version) -> Result<()> {
    match get_bin_version(plugin_executable) {
        Ok(version) => {
            info!(
                "{:>4}- {} - {}",
                "",
                bold(plugin),
                format_version_comparison(&version, latest_version)
            );
        }
        Err(err) => {
            info!(
                "{:>4}- {} - Error getting version string: {}",
                "",
                plugin,
                err.to_string()
            );
        }
    }
    Ok(())
}

fn check_fuelup() -> Result<()> {
    let fuelup_version: Version = Version::parse(clap::crate_version!())?;

    if let Ok(fuelup_download_cfg) = DownloadCfg::new(
        component::FUELUP,
        TargetTriple::from_component(component::FUELUP)?,
        None,
    ) {
        info!(
            "{} - {}",
            bold(component::FUELUP),
            format_version_comparison(&fuelup_version, &fuelup_download_cfg.version)
        );
    } else {
        error!("Failed to create DownloadCfg for component 'fuelup'; skipping check for 'fuelup'");
    }
    Ok(())
}

fn check_toolchain(toolchain: &str, verbose: bool) -> Result<()> {
    let description = DistToolchainDescription::from_str(toolchain)?;

    let dist_channel = Channel::from_dist_channel(&description)?;
    let latest_package_versions = collect_package_versions(dist_channel);

    let toolchain = Toolchain::new(toolchain)?;

    info!("{}", bold(&toolchain.name));

    for component in Components::collect_exclude_plugins()? {
        if let Some(latest_version) = latest_package_versions.get(&component.name) {
            let component_executable = toolchain.bin_path.join(&component.name);
            match get_bin_version(&component_executable) {
                Ok(version) => {
                    info!(
                        "{:>2}{} - {}",
                        "",
                        bold(&component.name),
                        format_version_comparison(&version, latest_version)
                    );
                }
                Err(e) => {
                    error!(
                        "{:>2}{} - Error getting version string: {}",
                        "",
                        bold(&component.name),
                        e.to_string()
                    );
                }
            }
            if verbose && component.name == component::FORC {
                for plugin in component::Components::collect_plugins()? {
                    if !plugin.is_main_executable() {
                        info!("{:>4}- {}", "", bold(&plugin.name));
                    }

                    for (index, executable) in plugin.executables.iter().enumerate() {
                        let plugin_executable = toolchain.bin_path.join(executable);

                        let mut plugin_name = &plugin.name;

                        if !plugin.is_main_executable() {
                            print!("{:>2}", "");
                            plugin_name = &plugin.executables[index];
                        }

                        let maybe_latest_version = plugin.publish.map_or_else(
                            || latest_package_versions.get(component::FORC),
                            |_| latest_package_versions.get(plugin_name),
                        );

                        if let Some(latest_version) = maybe_latest_version {
                            check_plugin(&plugin_executable, plugin_name, latest_version)?;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

pub fn check(command: CheckCommand) -> Result<()> {
    let CheckCommand { verbose } = command;

    let cfg = Config::from_env()?;

    for toolchain in cfg.list_dist_toolchains()? {
        check_toolchain(&toolchain, verbose)?;
    }

    check_fuelup()?;
    Ok(())
}
