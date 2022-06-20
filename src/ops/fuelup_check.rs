use std::collections::HashMap;
use std::io::Write;

use crate::{
    commands::check::CheckCommand,
    component::SUPPORTED_PLUGINS,
    config::Config,
    fmt::{bold, with_color_maybe_bold},
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
                    .collect::<Vec<&str>>()[1],
            )?;

            bold(|s| write!(s, "    - {} - ", plugin))?;
            if &version == latest_version {
                with_color_maybe_bold(|s| write!(s, "Up to date"), Color::Green, true)?;
                println!(": {}", version);
            } else {
                with_color_maybe_bold(|s| write!(s, "Update available "), Color::Green, true)?;
                println!("{} -> {}", version, latest_version);
            }
        }
        Err(e) => {
            bold(|s| write!(s, "    - {} - ", plugin))?;
            if plugin_executable.exists() {
                println!("execution error - {}", e);
            } else {
                println!("not found");
            }
        }
    }
    Ok(())
}

pub fn check(command: CheckCommand) -> Result<()> {
    let CheckCommand { verbose } = command;

    let cfg = Config::from_env()?;
    let toolchains = cfg.list_toolchains()?;
    let mut latest_versions: HashMap<String, Version> = HashMap::new();
    let components = Vec::from([component::FORC, component::FUEL_CORE, component::FUELUP]);

    for component in components {
        let download_cfg: DownloadCfg = DownloadCfg::new(component, None)?;
        latest_versions.insert(component.to_string(), download_cfg.version);
    }

    for toolchain in toolchains {
        if let Ok(toolchain) = Toolchain::from(&toolchain) {
            bold(|s| writeln!(s, "{}", &toolchain.name))?;
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
                                .collect::<Vec<&str>>()[1],
                        )?;

                        bold(|s| write!(s, "  {} - ", &component))?;
                        if version == latest_versions[component] {
                            with_color_maybe_bold(
                                |s| writeln!(s, "Up to date : {}", version),
                                Color::Green,
                                true,
                            )?;
                        } else {
                            with_color_maybe_bold(
                                |s| write!(s, "Update available"),
                                Color::Yellow,
                                true,
                            )?;
                            println!(" : {} -> {}", version, latest_versions[component]);
                        }
                    }
                    Err(e) => {
                        bold(|s| write!(s, "  {} - ", &component))?;
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
    }

    match std::process::Command::new(component::FUELUP)
        .arg("--version")
        .output()
    {
        Ok(o) => {
            let version = Version::parse(
                String::from_utf8_lossy(&o.stdout)
                    .split_whitespace()
                    .collect::<Vec<&str>>()[1],
            )?;

            bold(|s| write!(s, "{} - ", component::FUELUP))?;
            if version == latest_versions[component::FUELUP] {
                with_color_maybe_bold(|s| write!(s, "Up to date "), Color::Green, true)?;
                println!(": {}", version);
            } else {
                with_color_maybe_bold(|s| write!(s, "Update available : "), Color::Yellow, true)?;
                println!("{} -> {}", version, latest_versions[component::FUELUP]);
            }
        }
        Err(e) => {
            // Unclear how we might run into this if we run it from fuelup - print errors anyway
            bold(|s| write!(s, "  {} - ", component::FUELUP))?;
            println!("execution error - {}", e);
        }
    };
    Ok(())
}
