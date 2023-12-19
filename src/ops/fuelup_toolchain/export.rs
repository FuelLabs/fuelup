use crate::{
    commands::toolchain::ExportCommand,
    constants::{FUEL_TOOLCHAIN_TOML_FILE, VALID_CHANNEL_STR},
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
    let ExportCommand {
        name,
        channel,
        force,
    } = command;
    let mut toolchain_info_path = PathBuf::from("./").join(FUEL_TOOLCHAIN_TOML_FILE);
    if let Some(path) = get_fuel_toolchain_toml() {
        toolchain_info_path = path;
        if force {
            println_warning(&format!(
                "Because the `--force` argument was supplied, this file will be overwritten: {}",
                &toolchain_info_path.display(),
            ));
            if fs::remove_file(&toolchain_info_path).is_err() {
                bail!("failed to remove file {}", &toolchain_info_path.display());
            }
        } else {
            println_warning(&format!(
                "There is an existing toolchain override file at {}. \
                Do you wish to replace it with a new one? (y/N) ",
                &toolchain_info_path.display(),
            ));
            let mut need_replace = String::new();
            if reader.read_line(&mut need_replace).is_err() {
                bail!("failed to read user input");
            }
            if need_replace.trim() == "y" {
                if fs::remove_file(&toolchain_info_path).is_err() {
                    bail!("failed to remove file {}", &toolchain_info_path.display());
                };
            } else {
                bail!(
                    "Failed to export toolchain \
                    because a toolchain override file already exists at {}.",
                    &toolchain_info_path.display(),
                );
            }
        }
    }
    let toolchain_name = name.unwrap_or(Toolchain::from_settings()?.name);

    let export_toolchain = match DistToolchainDescription::from_str(&toolchain_name) {
        Ok(desc) => Toolchain::from_path(&desc.to_string()),
        Err(_) => Toolchain::from_path(&toolchain_name),
    };

    let mut version_map: HashMap<String, Version> = HashMap::new();
    for component in Components::collect_exclude_plugins()? {
        let _ = exec_version(&export_toolchain.bin_path.join(&component.name))
            .map(|version| version_map.insert(component.name.clone(), version));

        if component.name == component::FORC {
            let forc_executables = component.executables;
            for plugin in Components::collect_plugins()? {
                if !forc_executables.contains(&plugin.name) {
                    plugin.executables.iter().for_each(|executable| {
                        let _ = exec_version(&export_toolchain.bin_path.join(executable))
                            .map(|version| version_map.insert(executable.clone(), version));
                    });
                }
            }
        }
    }
    let mut export_channel = channel.unwrap_or(toolchain_name.clone());
    if toolchain_override::Channel::from_str(&export_channel).is_err() {
        println_warning(&format!(
            "Invalid channel '{}', expected one of {}. \
            Please input a valid channel: ",
            export_channel, VALID_CHANNEL_STR,
        ));
        let mut input_channel_name = String::new();
        if reader.read_line(&mut input_channel_name).is_err() {
            bail!("failed to read user input");
        }
        input_channel_name = String::from(input_channel_name.trim());
        if toolchain_override::Channel::from_str(&input_channel_name).is_err() {
            bail!(
                "Invalid channel '{}', expected one of {}.",
                input_channel_name,
                VALID_CHANNEL_STR,
            );
        } else {
            export_channel = input_channel_name;
        }
    }

    let toolchain_override = ToolchainOverride {
        cfg: OverrideCfg::new(
            ToolchainCfg {
                // here shouldn't be err as we has checked above
                channel: toolchain_override::Channel::from_str(&export_channel)?,
            },
            Some(version_map),
        ),
        path: toolchain_info_path.clone(),
    };
    let document = toolchain_override.to_toml();
    if std::fs::write(toolchain_override.path, document.to_string()).is_err() {
        bail!("failed to write {}", toolchain_info_path.display());
    }

    info!(
        "exported '{}' into '{}'",
        toolchain_name,
        toolchain_info_path.display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        channel, path::settings_file, toolchain::DistToolchainName,
        toolchain_override::ToolchainOverride,
    };
    use serial_test::serial;
    use std::{fs, io::Write};

    use super::*;

    // simulate input
    const INPUT_NOP: &[u8; 1] = b"\n";
    const INPUT_YES: &[u8; 2] = b"y\n";
    const INPUT_NO: &[u8; 2] = b"n\n";
    const INPUT_INVALID_CHANNEL: &[u8; 11] = b"my-channel\n";
    const INVALID_CHANNEL: &str = "my-channel";

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
                fs::create_dir_all(fuel_toolchain_toml_file.parent().unwrap()).unwrap();
                fs::File::create(&fuel_toolchain_toml_file).unwrap();
            }
        } else {
            fs::File::create(FUEL_TOOLCHAIN_TOML_FILE).unwrap();
        }
    }
    // mock setting.toml
    fn create_toolchain_settings_file() {
        let setting_file_path = settings_file();
        if !setting_file_path.exists() {
            fs::create_dir_all(setting_file_path.parent().unwrap()).unwrap();
            let mut file = fs::File::create(&setting_file_path).unwrap();
            file.write_all(b"default_toolchain = \"latest\"").unwrap();
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
    #[serial]
    #[should_panic]
    fn export_toolchain_with_exists_toolchain_info_throws_err() {
        create_toolchain_settings_file();
        create_toolchain_info();
        export(
            ExportCommand {
                name: Some(DistToolchainName::Beta3.to_string()),
                channel: None,
                force: false,
            },
            &INPUT_NO[..],
        )
        .unwrap();
    }

    #[test]
    #[serial]
    #[should_panic]
    fn export_toolchain_with_invalid_channel_provided_throws_err() {
        create_toolchain_settings_file();
        remove_toolchain_info();
        export(
            ExportCommand {
                name: Some(DistToolchainName::Beta3.to_string()),
                channel: Some(INVALID_CHANNEL.to_string()),
                force: false,
            },
            &INPUT_INVALID_CHANNEL[..],
        )
        .unwrap();
    }

    #[test]
    #[serial]
    #[should_panic]
    fn export_toolchain_with_invalid_channel_inputted_throws_err() {
        create_toolchain_settings_file();
        remove_toolchain_info();
        export(
            ExportCommand {
                name: Some(DistToolchainName::Latest.to_string()),
                channel: None,
                force: false,
            },
            &INPUT_INVALID_CHANNEL[..],
        )
        .unwrap();
    }

    #[test]
    #[serial]
    fn export_toolchain_with_exists_toolchain_info() {
        create_toolchain_settings_file();
        create_toolchain_info();

        // case: path exist with valid channel provided and with --force
        let channel = channel::BETA_3.to_string();
        export(
            ExportCommand {
                name: Some(DistToolchainName::Latest.to_string()),
                channel: Some(channel.clone()),
                force: true,
            },
            &INPUT_NOP[..],
        )
        .unwrap();
        check_toolchain_info_with_channel(&channel).unwrap();

        // case: path exist with valid channel inputted and with --force
        let channel = channel::BETA_3.to_string();
        let channel_input = format!("{}\n", channel);
        export(
            ExportCommand {
                name: Some(DistToolchainName::Latest.to_string()),
                channel: None,
                force: true,
            },
            channel_input.as_bytes(),
        )
        .unwrap();
        check_toolchain_info_with_channel(&channel).unwrap();

        // case: path exist with valid channel and without --force and input[yes]
        let channel = channel::BETA_3.to_string();
        export(
            ExportCommand {
                name: Some(DistToolchainName::Latest.to_string()),
                channel: Some(channel.clone()),
                force: false,
            },
            &INPUT_YES[..],
        )
        .unwrap();
        check_toolchain_info_with_channel(&channel).unwrap();

        // case: path exist with invalid channel and with --force and input valid channel
        let channel = channel::BETA_3.to_string();
        let channel_input = format!("{}\n", channel);
        export(
            ExportCommand {
                name: Some(DistToolchainName::Latest.to_string()),
                channel: Some(INVALID_CHANNEL.to_string()),
                force: true,
            },
            channel_input.as_bytes(),
        )
        .unwrap();
        check_toolchain_info_with_channel(&channel).unwrap();
    }

    #[test]
    #[serial]
    fn export_toolchain_without_exists_toolchain_info() {
        create_toolchain_settings_file();
        // case: path not exist with valid channel
        remove_toolchain_info();
        let channel = channel::BETA_3.to_string();
        export(
            ExportCommand {
                name: Some(DistToolchainName::Latest.to_string()),
                channel: Some(channel.clone()),
                force: false,
            },
            &INPUT_NOP[..],
        )
        .unwrap();
        check_toolchain_info_with_channel(&channel).unwrap();

        // case: path not exist with invalid channel and input valid channel
        remove_toolchain_info();
        let channel = channel::BETA_3.to_string();
        let channel_input = format!("{}\n", channel);
        export(
            ExportCommand {
                name: Some(DistToolchainName::Latest.to_string()),
                channel: None,
                force: false,
            },
            channel_input.as_bytes(),
        )
        .unwrap();
        check_toolchain_info_with_channel(&channel).unwrap();
    }
}
