use crate::{
    commands::toolchain::ExportCommand,
    constants::FUEL_TOOLCHAIN_TOML_FILE,
    path::get_fuel_toolchain_toml,
    toolchain::{DistToolchainDescription, Toolchain},
    toolchain_override::{self, OverrideCfg, ToolchainCfg, ToolchainOverride},
    util::version::exec_version,
};
use anyhow::{bail, Result};
use component::{self, Components};
use forc_tracing::println_warning;
use semver::Version;
use std::{collections::HashMap, path::PathBuf};
use std::{fs, io::BufRead, str::FromStr};
use tracing::info;

pub fn export(command: ExportCommand, mut reader: impl BufRead) -> Result<()> {
    let ExportCommand { name, force } = command;
    let mut toolchain_info_path = PathBuf::from("./").join(FUEL_TOOLCHAIN_TOML_FILE);
    if let Some(path) = get_fuel_toolchain_toml() {
        toolchain_info_path = path;
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
    if DistToolchainDescription::from_str(&toolchain_name).is_err() {
        println_warning(&format!(
            "Invalid channel '{}', expected one of <latest-YYYY-MM-DD|nightly-YYYY-MM-DD|beta-1|beta-2|beta-3|beta-4>. \
            Please input a valid channel: ",
            toolchain_name,
        ));
        let mut input_toolchain_name = String::new();
        reader.read_line(&mut input_toolchain_name).unwrap();
        input_toolchain_name = String::from(input_toolchain_name.trim());
        if DistToolchainDescription::from_str(&input_toolchain_name).is_err() {
            bail!(
                "Invalid channel '{}', expected one of <latest-YYYY-MM-DD|nightly-YYYY-MM-DD|beta-1|beta-2|beta-3|beta-4>.",
                input_toolchain_name,
            );
        } else {
            toolchain_name = input_toolchain_name;
        }
    }

    let toolchain_override = ToolchainOverride {
        cfg: OverrideCfg::new(
            ToolchainCfg {
                channel: toolchain_override::Channel::from_str(&toolchain_name).unwrap(),
            },
            Some(version_map),
        ),
        path: toolchain_info_path.clone(),
    };
    let document = toolchain_override.to_toml();
    std::fs::write(toolchain_override.path, document.to_string())
        .unwrap_or_else(|_| panic!("failed to write {}", toolchain_info_path.display()));

    info!(
        "exported '{}' into '{}'",
        toolchain_name,
        toolchain_info_path.display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{channel::BETA_3, toolchain_override::ToolchainOverride};
    use std::fs;

    use super::*;

    // simulate input
    const INPUT_NOP: &[u8; 1] = b"\n";
    const INPUT_YES: &[u8; 2] = b"y\n";
    const INPUT_NO: &[u8; 2] = b"n\n";
    const INPUT_INVALID_CHANNEL: &[u8; 10] = b"my-custom\n";
    const INVALID_CHANNEL: &str = "my-custom";

    fn remove_toolchain_info() {
        if let Some(fuel_toolchain_toml_file) = get_fuel_toolchain_toml() {
            if fuel_toolchain_toml_file.exists() {
                fs::remove_file(fuel_toolchain_toml_file).unwrap();
            }
        }
    }
    fn create_toolchain_info() {
        if let Some(fuel_toolchain_toml_file) = get_fuel_toolchain_toml() {
            if !fuel_toolchain_toml_file.exists() {
                fs::File::create(fuel_toolchain_toml_file).unwrap();
            }
        }
    }
    fn check_toolchain_info_with_channel(channel_name: &String) -> Result<()> {
        let toolchain_override = ToolchainOverride::from_project_root().unwrap();
        if toolchain_override
            .cfg
            .toolchain
            .channel
            .to_string()
            .eq(channel_name)
        {
            Ok(())
        } else {
            bail!("unexpected channel");
        }
    }

    #[test]
    #[should_panic]
    fn export_toolchain_with_exists_toolchain_info_throws_err() {
        create_toolchain_info();
        export(
            ExportCommand {
                name: Some(BETA_3.to_string()),
                force: false,
            },
            &INPUT_NO[..],
        )
        .unwrap();
    }

    #[test]
    #[should_panic]
    fn export_toolchain_with_invalid_channel_throws_err() {
        remove_toolchain_info();
        export(
            ExportCommand {
                name: Some(INVALID_CHANNEL.to_string()),
                force: false,
            },
            &INPUT_INVALID_CHANNEL[..],
        )
        .unwrap();
    }

    #[test]
    fn export_toolchain_with_exists_toolchain_info() {
        create_toolchain_info();

        // case: path exist with valid channel and with --force
        let channel = BETA_3.to_string();
        export(
            ExportCommand {
                name: Some(channel.to_string()),
                force: true,
            },
            &INPUT_NOP[..],
        )
        .unwrap();
        check_toolchain_info_with_channel(&channel).unwrap();

        // case: path exist with valid channel and without --force and input[yes]
        let channel = BETA_3.to_string();
        export(
            ExportCommand {
                name: Some(channel.to_string()),
                force: false,
            },
            &INPUT_YES[..],
        )
        .unwrap();
        check_toolchain_info_with_channel(&channel).unwrap();

        // case: path exist with invalid channel and with --force and input valid channel
        let channel = BETA_3.to_string();
        let channel_input = format!("{}\n", channel);
        export(
            ExportCommand {
                name: Some(INVALID_CHANNEL.to_string()),
                force: true,
            },
            channel_input.as_bytes(),
        )
        .unwrap();
        check_toolchain_info_with_channel(&channel).unwrap();
    }

    #[test]
    fn export_toolchain_without_exists_toolchain_info() {
        // case: path not exist with valid channel
        remove_toolchain_info();
        let channel = BETA_3.to_string();
        export(
            ExportCommand {
                name: Some(channel.to_string()),
                force: false,
            },
            &INPUT_NOP[..],
        )
        .unwrap();
        check_toolchain_info_with_channel(&channel).unwrap();

        // case: path not exist with invalid channel and input valid channel
        remove_toolchain_info();
        let channel = BETA_3.to_string();
        let channel_input = format!("{}\n", channel);
        export(
            ExportCommand {
                name: Some(INVALID_CHANNEL.to_string()),
                force: false,
            },
            channel_input.as_bytes(),
        )
        .unwrap();
        check_toolchain_info_with_channel(&channel).unwrap();
    }
}
