use crate::{
    channel::{Channel, PackageVersion},
    commands::check::CheckCommand,
    component::SUPPORTED_PLUGINS,
    config::Config,
    constants::{CHANNEL_LATEST_FILE_NAME, CHANNEL_NIGHTLY_FILE_NAME},
    file::read_file,
    fmt::{bold, colored_bold},
    path::fuelup_dir,
    target_triple::TargetTriple,
    toolchain::{DistToolchainName, OfficialToolchainDescription, Toolchain},
};
use anyhow::Result;
use semver::Version;
use std::collections::HashMap;
use std::io::Write;
use std::str::FromStr;
use std::{
    cmp::Ordering::{Equal, Greater, Less},
    path::Path,
};
use tempfile::tempdir_in;
use termcolor::Color;
use tracing::error;

use crate::{component, download::DownloadCfg};

fn collect_package_versions(channel: Channel) -> HashMap<String, PackageVersion> {
    let mut latest_versions: HashMap<String, PackageVersion> = HashMap::new();

    for (name, package) in channel.pkg.into_iter() {
        latest_versions.insert(name, package.version.clone());
    }

    latest_versions
}

fn compare_and_print_versions(
    current_version: &PackageVersion,
    latest_version: &PackageVersion,
) -> Result<()> {
    match current_version.cmp(latest_version) {
        Less => {
            colored_bold(Color::Yellow, |s| write!(s, "Update available"));
            println!(" : {} -> {}", current_version, latest_version);
        }
        Equal => {
            colored_bold(Color::Green, |s| write!(s, "Up to date"));
            println!(" : {}", current_version);
        }
        Greater => {
            print!(" : {}", current_version);
            colored_bold(Color::Yellow, |s| write!(s, " (unstable)"));
            print!(" -> {}", latest_version);
            colored_bold(Color::Green, |s| writeln!(s, " (recommended)"));
        }
    }
    Ok(())
}

fn check_plugin(
    plugin_executable: &Path,
    plugin: &str,
    current_version: &PackageVersion,
    latest_version: &PackageVersion,
) -> Result<()> {
    if plugin_executable.is_file() {
        print!("    - ");
        bold(|s| write!(s, "{}", plugin));
        print!(" - ");
        compare_and_print_versions(current_version, latest_version)?;
    } else {
        print!("  ");
        bold(|s| write!(s, "{}", &plugin));
        println!(" - not installed");
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
        bold(|s| write!(s, "{} - ", component::FUELUP));
        compare_and_print_versions(
            &PackageVersion {
                semver: fuelup_version,
                date: None,
            },
            &fuelup_download_cfg.version,
        )?;
    } else {
        error!("Failed to create DownloadCfg for component 'fuelup'; skipping check for 'fuelup'");
    }
    Ok(())
}

fn check_toolchain(toolchain: &str, verbose: bool) -> Result<()> {
    let description = OfficialToolchainDescription::from_str(toolchain)?;

    let fuelup_dir = fuelup_dir();
    let tmp_dir = tempdir_in(&fuelup_dir)?;

    let dist_channel = Channel::from_dist_channel(&description, tmp_dir.into_path())?;
    let latest_package_versions = collect_package_versions(dist_channel);

    let toolchain = Toolchain::new(toolchain)?;

    let channel_file_name = match description.name {
        DistToolchainName::Latest => CHANNEL_LATEST_FILE_NAME,
        DistToolchainName::Nightly => CHANNEL_NIGHTLY_FILE_NAME,
    };
    let toml_path = toolchain.path.join(channel_file_name);
    let toml = read_file("channel", &toml_path)?;
    let channel = Channel::from_toml(&toml)?;

    bold(|s| writeln!(s, "{}", &toolchain.name));

    for component in [component::FORC, component::FUEL_CORE] {
        let version = &channel.pkg[component].version;
        let latest_version = &latest_package_versions[component];

        let component_executable = toolchain.bin_path.join(component);

        if component_executable.is_file() {
            bold(|s| write!(s, "  {} - ", &component));
            compare_and_print_versions(version, latest_version)?;
        } else {
            print!("  ");
            bold(|s| write!(s, "{}", &component));
            println!(" - not installed");
        }

        if verbose && component == component::FORC {
            for plugin in SUPPORTED_PLUGINS {
                if plugin == &component::FORC_DEPLOY {
                    bold(|s| writeln!(s, "    - forc-client"));
                }
                if plugin == &component::FORC_RUN || plugin == &component::FORC_DEPLOY {
                    print!("  ");
                }

                let plugin_executable = toolchain.bin_path.join(&plugin);
                check_plugin(&plugin_executable, plugin, version, latest_version)?;
            }
        }
    }
    Ok(())
}

pub fn check(command: CheckCommand) -> Result<()> {
    let CheckCommand { verbose } = command;

    let cfg = Config::from_env()?;

    for toolchain in cfg.list_official_toolchains()? {
        // TODO: remove once date/target are supported
        let name = toolchain.split_once('-').unwrap_or_default().0;
        check_toolchain(name, verbose)?;
    }

    check_fuelup()?;
    Ok(())
}
