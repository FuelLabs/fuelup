use crate::file::get_bin_version;
use crate::fmt::bold;
use crate::store::Store;
use crate::{
    config::Config,
    fmt::print_header,
    path::fuelup_dir,
    target_triple::TargetTriple,
    toolchain::{DistToolchainDescription, Toolchain},
    toolchain_override::ToolchainOverride,
};
use anyhow::Result;
use component::{self, Components};
use semver::Version;
use std::collections::HashMap;
use std::str::FromStr;
use tracing::info;

pub fn show() -> Result<()> {
    info!("{}: {}", bold("Default host"), TargetTriple::from_host()?);
    info!("{}: {}", bold("fuelup home"), fuelup_dir().display());

    print_header("Installed toolchains");
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

    print_header("active toolchain");
    info!("{}", active_toolchain_message);

    let mut version_map: HashMap<String, Version> = HashMap::new();
    for component in Components::collect_exclude_plugins()? {
        let component_executable = active_toolchain.bin_path.join(&component.name);
        let version_text: String = match get_bin_version(component_executable.as_path()) {
            Ok(version) => {
                version_map.insert(component.name.clone(), version.clone());
                format!("{}", version)
            }
            Err(e) => e.to_string(),
        };

        info!("{:>2}{} : {}", "", bold(&component.name), version_text);

        if component.name == component::FORC {
            for plugin in Components::collect_plugins()? {
                if !plugin.is_main_executable() {
                    info!("{:>4}- {}", "", bold(&plugin.name));

                    for executable in plugin.executables.iter() {
                        let plugin_executable = active_toolchain.bin_path.join(executable);
                        let version_text = match get_bin_version(plugin_executable.as_path()) {
                            Ok(version) => {
                                version_map.insert(executable.clone(), version.clone());
                                format!("{}", version)
                            }
                            Err(e) => e.to_string(),
                        };
                        info!("{:>6}- {} : {}", "", bold(executable), version_text);
                    }
                } else {
                    let plugin_executable = active_toolchain.bin_path.join(&plugin.name);
                    let version_text = match get_bin_version(plugin_executable.as_path()) {
                        Ok(version) => {
                            version_map.insert(plugin.name.clone(), version.clone());
                            format!("{}", version)
                        }
                        Err(e) => e.to_string(),
                    };
                    info!("{:>4}- {} : {}", "", bold(&plugin.name), version_text);
                }
            }
        }
    }

    let store = Store::from_env()?;

    let mut fuels_version_header_shown = false;
    for component in Components::collect_show_fuels_versions()? {
        if let Some(version) = version_map.get(&component.name) {
            if let Ok(fuels_version) = store.get_cached_fuels_version(&component.name, version) {
                // Only print the header if we find an Ok fuels_version to show.
                if !fuels_version_header_shown {
                    print_header("fuels versions");
                    fuels_version_header_shown = true;
                }
                info!("{} : {}", bold(&component.name), fuels_version);
            };
        }
    }

    Ok(())
}
