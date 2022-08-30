use crate::{
    channel::Channel,
    commands::check::CheckCommand,
    component::SUPPORTED_PLUGINS,
    config::Config,
    constants::{CHANNEL_LATEST_FILE_NAME, CHANNEL_NIGHTLY_FILE_NAME, DATE_FORMAT},
    file::read_file,
    fmt::{bold, colored_bold},
    path::fuelup_dir,
    target_triple::TargetTriple,
    toolchain::{DistToolchainName, OfficialToolchainDescription, Toolchain},
};
use anyhow::Result;
use semver::Version;
use std::cmp::Ordering::{Equal, Greater, Less};
use std::collections::HashMap;
use std::io::Write;
use std::str::FromStr;
use tempfile::tempdir_in;
use termcolor::Color;
use time::Date;
use tracing::error;

use crate::{component, download::DownloadCfg};

fn collect_versions(channel: Channel) -> HashMap<String, String> {
    let mut latest_versions: HashMap<String, String> = HashMap::new();
    for (name, package) in channel.pkg.into_iter() {
        latest_versions.insert(name, package.version);
    }

    latest_versions
}

fn compare_and_print_versions(
    toolchain: DistToolchainName,
    current_version_string: String,
    target_version_string: String,
) -> Result<()> {
    let (current_version, target_version) = match toolchain {
        DistToolchainName::Latest => (
            Version::parse(&current_version_string)?,
            Version::parse(&target_version_string)?,
        ),
        DistToolchainName::Nightly => (
            // 0.11.0-nightly
            Version::parse(&current_version_string.split_once('-').unwrap_or_default().0)?,
            Version::parse(&target_version_string.split_once('-').unwrap_or_default().0)?,
        ),
    };
    match current_version.cmp(&target_version) {
        Less => {
            colored_bold(Color::Yellow, |s| write!(s, "Update available"));
            println!(" : {} -> {}", current_version, target_version);
        }
        Equal => {
            if toolchain == DistToolchainName::Nightly {
                // dates always come in the format '(YYYY-MM-DD)'
                let current_date = Date::parse(
                    current_version_string
                        .split_once('(')
                        .unwrap_or_default()
                        .1
                        .trim_end_matches(')'),
                    DATE_FORMAT,
                )?;
                let target_date = Date::parse(
                    target_version_string
                        .split_once('(')
                        .unwrap_or_default()
                        .1
                        .trim_end_matches(')'),
                    DATE_FORMAT,
                )?;

                match current_date.cmp(&target_date) {
                    Less => {
                        colored_bold(Color::Yellow, |s| write!(s, "Update available"));
                        println!(" : {} -> {}", current_version_string, target_version_string);
                    }
                    Equal => {
                        colored_bold(Color::Green, |s| write!(s, "Up to date"));
                        println!(" : {}", current_version_string);
                    }
                    Greater => {
                        print!(" : {}", current_version_string);
                        colored_bold(Color::Yellow, |s| write!(s, " (unstable)"));
                        print!(" -> {}", target_version_string);
                        colored_bold(Color::Green, |s| writeln!(s, " (recommended)"));
                    }
                }
            } else {
                colored_bold(Color::Green, |s| write!(s, "Up to date"));
                println!(" : {}", current_version);
            }
        }
        Greater => {
            print!(" : {}", current_version);
            colored_bold(Color::Yellow, |s| write!(s, " (unstable)"));
            print!(" -> {}", target_version);
            colored_bold(Color::Green, |s| writeln!(s, " (recommended)"));
        }
    }
    Ok(())
}

fn check_plugin(
    toolchain: &Toolchain,
    description: &OfficialToolchainDescription,
    plugin: &str,
    current_version: String,
    latest_version: String,
) -> Result<()> {
    let plugin_executable = toolchain.bin_path.join(&plugin);
    if plugin_executable.is_file() {
        print!("    - ");
        bold(|s| write!(s, "{}", plugin));
        print!(" - ");
        compare_and_print_versions(
            DistToolchainName::from_str(&description.name.to_string())?,
            current_version,
            latest_version,
        )?;
    } else {
        print!("  ");
        bold(|s| write!(s, "{}", &plugin));
        println!(" - not installed");
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
            DistToolchainName::Latest,
            FUELUP_VERSION.to_string(),
            fuelup_download_cfg.version,
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
    let latest_versions = match Channel::from_dist_channel(&description, tmp_dir.into_path()) {
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

    let channel_file_name = match description.name {
        DistToolchainName::Latest => CHANNEL_LATEST_FILE_NAME,
        DistToolchainName::Nightly => CHANNEL_NIGHTLY_FILE_NAME,
    };
    let toml_path = toolchain.path.join(channel_file_name);
    let toml = read_file(CHANNEL_LATEST_FILE_NAME, &toml_path)?;
    let channel = Channel::from_toml(&toml)?;

    bold(|s| writeln!(s, "{}", &toolchain.name));

    for component in [component::FORC, component::FUEL_CORE] {
        let version = &channel.pkg[component].version;
        let component_executable = toolchain.bin_path.join(component);

        if component_executable.is_file() {
            bold(|s| write!(s, "  {} - ", &component));
            compare_and_print_versions(
                DistToolchainName::from_str(&description.name.to_string())?,
                version.to_string(),
                latest_versions[component].to_string(),
            )?;
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
                check_plugin(
                    &toolchain,
                    &description,
                    plugin,
                    version.to_string(),
                    latest_versions[component::FORC].clone(),
                )?;
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
