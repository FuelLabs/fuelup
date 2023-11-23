use crate::{
    commands::toolchain::ExportCommand,
    toolchain::Toolchain,
    constants::FUEL_TOOLCHAIN_TOML_FILE,
    toolchain_override::{self, OverrideCfg, ToolchainCfg, ToolchainOverride},
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use semver::Version;
use anyhow::{bail, Result};
use component::{self, Components};
use tracing::info;

pub fn export(command: ExportCommand) -> Result<()> {
    let ExportCommand { name, force} = command;
    let path = PathBuf::from("./").join(FUEL_TOOLCHAIN_TOML_FILE);
    if !force && path.exists() {
        bail!("{FUEL_TOOLCHAIN_TOML_FILE} already exists");
    }
    let mut toolchain_name = Toolchain::from_settings()?.name;
    if let Some(name) = name {
        toolchain_name = name;
    }
    let export_toolchain = Toolchain::from_path(&toolchain_name);

    let mut version_map: HashMap<String, Version> = HashMap::new();
    for component in Components::collect_exclude_plugins()? {
        let component_executable = export_toolchain.bin_path.join(&component.name);
        if let Ok(version) = get_exec_version(component_executable.as_path()) {
            version_map.insert(component.name.clone(), version);
        };

        if component.name == component::FORC {
            let forc_executables = component.executables;
            for plugin in Components::collect_plugins()? {
                if !forc_executables.contains(&plugin.name) {
                    if !plugin.is_main_executable() {
                        for executable in plugin.executables.iter() {
                            let plugin_executable = export_toolchain.bin_path.join(executable);
                            if let Ok(version) = get_exec_version(plugin_executable.as_path()) {
                                version_map.insert(executable.clone(), version);
                            };
                        }
                    } else {
                        let plugin_executable = export_toolchain.bin_path.join(&plugin.name);
                        if let Ok(version) = get_exec_version(plugin_executable.as_path()) {
                            version_map.insert(plugin.name.clone(), version);
                        };
                    }
                }
            }
        }
    }

    let toolchain_override = ToolchainOverride {
        cfg: OverrideCfg::new(
            ToolchainCfg {
                channel: toolchain_override::Channel::from_str(&toolchain_name).unwrap(),
            },
            Some(version_map),
        ),
        path,
    };
    let document = toolchain_override.to_toml();
    std::fs::write(toolchain_override.path, document.to_string())
        .unwrap_or_else(|_| panic!("failed to write {FUEL_TOOLCHAIN_TOML_FILE}"));

    info!("exported '{toolchain_name}' into '{FUEL_TOOLCHAIN_TOML_FILE}'");
    Ok(())
}

fn get_exec_version(component_executable: &Path) -> Result<Version> {
    match std::process::Command::new(component_executable)
        .arg("--version")
        .output()
    {
        Ok(o) => {
            let output = String::from_utf8_lossy(&o.stdout).into_owned();
            match output.split_whitespace().last() {
                Some(v) => {
                    let version = Version::parse(v)?;
                    return Ok(version);
                }
                None => {
                    bail!("error getting version string");
                }
            };
        }
        Err(e) => {
            if component_executable.exists() {
                bail!("execute '{} --version' error - {}", component_executable.display(), e);
            }
        }
    }

    bail!("could not show version: {}", component_executable.display());
}