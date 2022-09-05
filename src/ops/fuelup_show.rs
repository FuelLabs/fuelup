use anyhow::Result;
use semver::Version;
use std::io::Write;
use std::str::FromStr;

use crate::{
    channel::Channel,
    component::{self, SUPPORTED_PLUGINS},
    config::Config,
    constants::{CHANNEL_LATEST_FILE_NAME, CHANNEL_NIGHTLY_FILE_NAME},
    file::read_file,
    fmt::{bold, print_header},
    path::fuelup_dir,
    target_triple::TargetTriple,
    toolchain::{DistToolchainName, OfficialToolchainDescription, Toolchain},
};

pub fn show() -> Result<()> {
    bold(|s| write!(s, "Default host: "));
    println!("{}", TargetTriple::from_host()?);

    bold(|s| write!(s, "fuelup home: "));
    println!("{}", fuelup_dir().display());
    println!();

    print_header("installed toolchains");
    let cfg = Config::from_env()?;
    let active_toolchain = Toolchain::from_settings()?;

    for toolchain in cfg.list_toolchains()? {
        if toolchain == active_toolchain.name {
            println!("{} (default)", toolchain);
        } else {
            println!("{}", toolchain);
        }
    }

    println!();
    print_header("active toolchain");

    println!("{} (default)", active_toolchain.name);

    let channel = if active_toolchain.is_official() {
        let description = OfficialToolchainDescription::from_str(
            active_toolchain.name.split_once('-').unwrap_or_default().0,
        )?;
        let channel_file_name = match description.name {
            DistToolchainName::Latest => CHANNEL_LATEST_FILE_NAME,
            DistToolchainName::Nightly => CHANNEL_NIGHTLY_FILE_NAME,
        };
        let toml_path = active_toolchain.path.join(channel_file_name);
        let toml = read_file("channel", &toml_path)?;
        Some(Channel::from_toml(&toml)?)
    } else {
        None
    };

    for component in [component::FORC, component::FUEL_CORE] {
        if let Some(c) = channel.as_ref() {
            let version = &c.pkg[component].version;
            bold(|s| write!(s, "  {}", &component));
            println!(" : {}", version);
        } else {
            let component_executable = active_toolchain.bin_path.join(component);
            match std::process::Command::new(&component_executable)
                .arg("--version")
                .output()
            {
                Ok(o) => {
                    let output = String::from_utf8_lossy(&o.stdout).into_owned();
                    match output.split_whitespace().nth(1) {
                        Some(v) => {
                            let version = Version::parse(v)?;
                            bold(|s| write!(s, "  {}", &component));
                            println!(" : {}", version);
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
            }
        };

        if component == component::FORC {
            for plugin in SUPPORTED_PLUGINS {
                if let Some(c) = channel.as_ref() {
                    let version = &c.pkg[component].version;

                    if plugin == &component::FORC_DEPLOY {
                        bold(|s| writeln!(s, "    - forc-client"));
                    }
                    if plugin == &component::FORC_RUN || plugin == &component::FORC_DEPLOY {
                        print!("  ");
                    }
                    bold(|s| write!(s, "    - {}", &plugin));
                    println!(" : {}", version);
                } else {
                    let plugin_executable = active_toolchain.bin_path.join(&plugin);
                    if plugin == &component::FORC_DEPLOY {
                        bold(|s| writeln!(s, "    - forc-client"));
                    }
                    if plugin == &component::FORC_RUN || plugin == &component::FORC_DEPLOY {
                        print!("  ");
                    }
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
                                    println!(" : {}", version);
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
                }
            }
        }
    }

    Ok(())
}
