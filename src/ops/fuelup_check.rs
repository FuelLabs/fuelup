use crate::{
    channel::Channel,
    commands::check::CheckCommand,
    component::Components,
    config::Config,
    fmt::{bold, colored_bold},
    path::fuelup_dir,
    target_triple::TargetTriple,
    toolchain::{OfficialToolchainDescription, Toolchain},
};
use anyhow::{bail, Result};
use semver::Version;
use std::io::Write;
use std::str::FromStr;
use std::{
    cmp::Ordering::{Equal, Greater, Less},
    path::Path,
};
use std::{collections::HashMap, process::Command};
use tempfile::tempdir_in;
use termcolor::Color;
use tracing::error;

use crate::{component, download::DownloadCfg};

fn collect_package_versions(channel: Channel) -> HashMap<String, Version> {
    let mut latest_versions: HashMap<String, Version> = HashMap::new();

    for (name, package) in channel.pkg.into_iter() {
        latest_versions.insert(name, package.version);
    }

    latest_versions
}

fn compare_and_print_versions(current_version: &Version, latest_version: &Version) -> Result<()> {
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
    current_version: &Version,
    latest_version: &Version,
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
        compare_and_print_versions(&fuelup_version, &fuelup_download_cfg.version)?;
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

    bold(|s| writeln!(s, "{}", &toolchain.name));

    for component in Components::collect_exclude_plugins()? {
        let component_executable = toolchain.bin_path.join(&component.name);
        let version = match Command::new(&component_executable)
            .arg("--version")
            .output()
        {
            Ok(o) => {
                let output = String::from_utf8_lossy(&o.stdout).into_owned();
                println!("{:?}", o);
                Version::parse(&output)?
            }
            Err(_) => bail!(""),
        };
        let latest_version = &latest_package_versions[&component.name];

        if component_executable.is_file() {
            bold(|s| write!(s, "  {} - ", &component.name));
            compare_and_print_versions(&version, latest_version)?;
        } else {
            print!("  ");
            bold(|s| write!(s, "{}", &component.name));
            println!(" - not installed");
        }

        if verbose && component.name == component::FORC {
            for plugin in component::Components::collect_plugins()? {
                if !plugin.is_main_executable() {
                    bold(|s| writeln!(s, "    - {}", plugin.name));
                }

                for (index, executable) in plugin.executables.iter().enumerate() {
                    let plugin_executable = toolchain.bin_path.join(executable);

                    let mut plugin_name = &plugin.name;
                    if !plugin.is_main_executable() {
                        print!("  ");
                        plugin_name = &plugin.executables[index];
                    }
                    check_plugin(&plugin_executable, plugin_name, &version, latest_version)?;
                }
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
