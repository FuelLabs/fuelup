use anyhow::Result;
use component::{self, Components};
use semver::Version;
use std::str::FromStr;
use std::{io::Write, path::Path};
use tracing::{error, info};

use crate::{
    config::Config,
    fmt::{bold, print_header},
    path::fuelup_dir,
    target_triple::TargetTriple,
    toolchain::{DistToolchainDescription, Toolchain},
    toolchain_override::ToolchainOverride,
};

fn exec_show_version(component_executable: &Path) -> Result<()> {
    match std::process::Command::new(component_executable)
        .arg("--version")
        .output()
    {
        Ok(o) => {
            let output = String::from_utf8_lossy(&o.stdout).into_owned();
            match output.split_whitespace().last() {
                Some(v) => {
                    let version = Version::parse(v)?;
                    info!(" : {}", version);
                }
                None => {
                    error!(" : Error getting version string");
                }
            };
        }
        Err(e) => {
            print!(" - ");
            if component_executable.exists() {
                error!("execution error - {}", e);
            } else {
                error!("not found");
            }
        }
    }

    Ok(())
}

pub fn show() -> Result<()> {
    bold(|s| write!(s, "Default host: "));
    info!("{}", TargetTriple::from_host()?);

    bold(|s| write!(s, "fuelup home: "));
    info!("{}\n", fuelup_dir().display());

    print_header("installed toolchains");
    let cfg = Config::from_env()?;
    let mut active_toolchain = Toolchain::from_settings()?;

    let toolchain_override = ToolchainOverride::from_project_root();

    let override_name = if let Some(toolchain_override) = toolchain_override.as_ref() {
        match DistToolchainDescription::from_str(
            &toolchain_override.cfg.toolchain.channel.to_string(),
        ) {
            Ok(desc) => Some(desc.to_string()),
            Err(_) => Some(toolchain_override.cfg.toolchain.channel.to_string()),
        }
    } else {
        None
    };

    for toolchain in cfg.list_toolchains()? {
        let mut message = toolchain.clone();
        if toolchain == active_toolchain.name {
            message.push_str(" (default)")
        }

        if Some(toolchain) == override_name {
            message.push_str(" (override)");
        }
        info!("{}", message)
    }

    let mut active_toolchain_message = String::new();
    if let Some(toolchain_override) = toolchain_override {
        // We know that the override exists, but we want to show the target triple as well.
        let override_name = override_name.as_ref().unwrap();

        let should_append_default = &active_toolchain.name == override_name;

        active_toolchain = Toolchain::from_path(override_name);
        active_toolchain_message.push_str(&format!("{} (override)", active_toolchain.name));
        if should_append_default {
            active_toolchain_message.push_str(" (default)");
        }

        active_toolchain_message
            .push_str(&format!(", path: {}", toolchain_override.path.display()));
    } else {
        active_toolchain_message.push_str(&format!("{} (default)", active_toolchain.name));
    };

    print_header("\nactive toolchain");
    info!("{}", active_toolchain_message);

    for component in Components::collect_exclude_plugins()? {
        bold(|s| write!(s, "  {}", &component.name));
        let component_executable = active_toolchain.bin_path.join(&component.name);
        exec_show_version(component_executable.as_path())?;

        if component.name == component::FORC {
            for plugin in Components::collect_plugins()? {
                bold(|s| write!(s, "    - {}", &plugin.name));
                if !plugin.is_main_executable() {
                    info!("");
                    for executable in plugin.executables.iter() {
                        bold(|s| write!(s, "      - {}", &executable));
                        let plugin_executable = active_toolchain.bin_path.join(executable);
                        exec_show_version(plugin_executable.as_path())?;
                    }
                } else {
                    let plugin_executable = active_toolchain.bin_path.join(&plugin.name);
                    exec_show_version(plugin_executable.as_path())?;
                }
            }
        }
    }

    Ok(())
}
