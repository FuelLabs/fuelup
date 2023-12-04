use crate::{
    commands::toolchain::ExportCommand,
    constants::FUEL_TOOLCHAIN_TOML_FILE,
    path::get_fuel_toolchain_toml,
    toolchain::Toolchain,
    toolchain_override::{self, OverrideCfg, ToolchainCfg, ToolchainOverride},
    util::version::exec_version,
};
use anyhow::{bail, Result};
use component::{self, Components};
use forc_tracing::println_warning;
use semver::Version;
use std::collections::HashMap;
use std::{
    fs,
    io::BufRead,
    str::FromStr,
};
use tracing::info;

pub fn export(command: ExportCommand, mut reader: impl BufRead) -> Result<()> {
    let ExportCommand { name, force } = command;
    let toolchain_info_path = get_fuel_toolchain_toml().unwrap();
    if toolchain_info_path.exists() {
        if force {
            println_warning(&format!(
                "Because the `--force` argument was supplied, the toolchain info file at {} will be removed.",
                &toolchain_info_path.display(),
            ));
            fs::remove_file(&toolchain_info_path).unwrap();
        } else {
            println_warning(&format!(
                "There is an existing toolchain info file at {}. \
                Do you wish to replace it with a new one? (y/N) ",
                &toolchain_info_path.display(),
            ));
            let mut need_replace = String::new();
            reader.read_line(&mut need_replace).unwrap();
            if need_replace.trim() == "y" {
                fs::remove_file(&toolchain_info_path).unwrap();
            } else {
                bail!(
                    "Failed to create a new toolchain info file at {} \
                    because a toolchain info file already exists at that location.",
                    &toolchain_info_path.display(),
                );
            }
        }
    }

    let mut toolchain_name = Toolchain::from_settings()?.name;
    if let Some(name) = name {
        toolchain_name = name;
    }
    let export_toolchain = Toolchain::from_path(&toolchain_name);

    let mut version_map: HashMap<String, Version> = HashMap::new();
    for component in Components::collect_exclude_plugins()? {
        let component_executable = export_toolchain.bin_path.join(&component.name);
        if let Ok(version) = exec_version(component_executable.as_path()) {
            version_map.insert(component.name.clone(), version);
        };

        if component.name == component::FORC {
            let forc_executables = component.executables;
            for plugin in Components::collect_plugins()? {
                if !forc_executables.contains(&plugin.name) {
                    if !plugin.is_main_executable() {
                        for executable in plugin.executables.iter() {
                            let plugin_executable = export_toolchain.bin_path.join(executable);
                            if let Ok(version) = exec_version(plugin_executable.as_path()) {
                                version_map.insert(executable.clone(), version);
                            };
                        }
                    } else {
                        let plugin_executable = export_toolchain.bin_path.join(&plugin.name);
                        if let Ok(version) = exec_version(plugin_executable.as_path()) {
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
        path: toolchain_info_path,
    };
    let document = toolchain_override.to_toml();
    std::fs::write(toolchain_override.path, document.to_string())
        .unwrap_or_else(|_| panic!("failed to write {FUEL_TOOLCHAIN_TOML_FILE}"));

    info!("exported '{toolchain_name}' into '{FUEL_TOOLCHAIN_TOML_FILE}'");
    Ok(())
}
