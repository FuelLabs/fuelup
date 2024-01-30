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
        match force {
            true => {
                println_warning(&format!(
                    "Because the `--force` argument was supplied, this file will be overwritten: {}",
                    &toolchain_info_path.display(),
                ));
                if fs::remove_file(&toolchain_info_path).is_err() {
                    bail!("Failed to remove file {}", &toolchain_info_path.display());
                }
            }
            false => {
                println_warning(&format!(
                    "There is an existing toolchain override file at {}. \
                    Do you wish to replace it with a new one? (y/N) ",
                    &toolchain_info_path.display(),
                ));
                let mut need_replace = String::new();
                if reader.read_line(&mut need_replace).is_err() {
                    bail!("Failed to read user input");
                }
                if need_replace.trim() == "y" {
                    if fs::remove_file(&toolchain_info_path).is_err() {
                        bail!("Failed to remove file {}", &toolchain_info_path.display());
                    };
                } else {
                    println_warning("Cancelled toolchain export");
                    return Ok(());
                }
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
            \nPlease input a valid channel: ",
            export_channel, VALID_CHANNEL_STR,
        ));
        let mut input_channel_name = String::new();
        if reader.read_line(&mut input_channel_name).is_err() {
            bail!("Failed to read user input");
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
        bail!("Failed to write {}", toolchain_info_path.display());
    }

    info!(
        "Exported toolchain \"{}\" to file: \"{}\"",
        toolchain_name,
        toolchain_info_path.display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        channel, path::settings_file, toolchain::DistToolchainName,
        toolchain_override::ToolchainOverride,
    };
    use pretty_assertions::assert_eq;
    use serial_test::serial;
    use std::fs;

    // Simulate user input
    const INPUT_NOP: &[u8; 1] = b"\n";
    const INPUT_YES: &[u8; 2] = b"y\r";
    const INPUT_NO: &[u8; 4] = b"n \r\n";
    const INPUT_INVALID_CHANNEL: &[u8; 11] = b"my-channel\n";
    const INVALID_CHANNEL: &str = "my-channel";

    // Mock file contents
    const NO_COMPONENTS_FILE_CONTENTS: &str = r#"
[toolchain]
channel = "beta-3"
"#;

    const BETA_3_FILE_CONTENTS: &str = r#"
[toolchain]
channel = "beta-3"

[components]
forc = "0.37.3"
forc-deploy = "0.37.3"
forc-explore = "0.28.1"
forc-run = "0.37.3"
forc-tx = "0.37.3"
forc-wallet = "0.2.2"
fuel-core = "0.17.11"
"#;

    fn remove_toolchain_info() {
        let toolchain_path = match get_fuel_toolchain_toml() {
            Some(path) => path,
            None => PathBuf::from(FUEL_TOOLCHAIN_TOML_FILE),
        };

        if toolchain_path.exists() {
            fs::remove_file(&toolchain_path).expect(&format!("remove file {:?}", toolchain_path));
        }
    }
    fn create_toolchain_info() {
        let toolchain_path = match get_fuel_toolchain_toml() {
            Some(path) => path,
            None => PathBuf::from(FUEL_TOOLCHAIN_TOML_FILE),
        };

        let _ =
            fs::create_dir_all(toolchain_path.parent().unwrap()).expect("create parent directory");

        std::fs::write(toolchain_path, NO_COMPONENTS_FILE_CONTENTS.to_string())
            .expect("write toolchain file");
    }

    fn assert_toolchain_info(expected_toml: &str) {
        let toolchain_path = get_fuel_toolchain_toml().expect("toolchain toml exists");
        assert!(toolchain_path.exists());
        let actual_toml = fs::read_to_string(&toolchain_path).unwrap();
        assert_eq!(expected_toml.to_string(), actual_toml);
    }

    // Creates a default `settings.toml`
    fn create_settings_file() {
        let setting_file_path = settings_file();
        if !setting_file_path.exists() {
            fs::create_dir_all(setting_file_path.parent().unwrap())
                .expect("create parent directory");
            std::fs::write(setting_file_path, b"default_toolchain = \"latest\"")
                .expect("write settings file");
        }
    }

    fn assert_channel_name(expected: String) {
        let toolchain_path = get_fuel_toolchain_toml().expect("toolchain toml exists");
        let toolchain_override =
            ToolchainOverride::from_path(toolchain_path).expect("toolchain override");
        let actual = toolchain_override.cfg.toolchain.channel.to_string();
        assert_eq!(expected, actual);
    }

    #[test]
    #[serial]
    fn test_export_toolchain_with_existing_toolchain_info_then_cancel() {
        create_settings_file();
        create_toolchain_info();

        export(
            ExportCommand {
                name: Some(DistToolchainName::Beta3.to_string()),
                channel: None,
                force: false,
            },
            &INPUT_NO[..],
        )
        .expect("should succeed");
        assert_toolchain_info(NO_COMPONENTS_FILE_CONTENTS);
    }

    #[test]
    #[serial]
    fn test_export_toolchain_with_forced_overwrite() {
        create_settings_file();
        create_toolchain_info();

        let channel = channel::BETA_3.to_string();
        export(
            ExportCommand {
                name: None,
                channel: Some(channel.clone()),
                force: true,
            },
            &INPUT_NOP[..],
        )
        .expect("should succeed");
        assert_channel_name(channel);
        assert_toolchain_info(BETA_3_FILE_CONTENTS);
    }

    #[test]
    #[serial]
    fn test_export_toolchain_with_forced_overwrite_and_valid_input() {
        create_settings_file();
        create_toolchain_info();

        let channel = channel::BETA_3.to_string();
        let channel_input = format!("{}\n", channel);
        export(
            ExportCommand {
                name: None,
                channel: None,
                force: true,
            },
            channel_input.as_bytes(),
        )
        .expect("should succeed");
        assert_channel_name(channel);
        assert_toolchain_info(BETA_3_FILE_CONTENTS);
    }

    #[test]
    #[serial]
    fn test_export_toolchain_with_forced_overwrite_and_valid_provide() {
        create_settings_file();
        create_toolchain_info();

        let channel = channel::BETA_3.to_string();
        let channel_input = format!("{}\n", channel);
        export(
            ExportCommand {
                name: None,
                channel: Some(INVALID_CHANNEL.to_string()),
                force: true,
            },
            channel_input.as_bytes(),
        )
        .expect("should succeed");
        assert_channel_name(channel);
        assert_toolchain_info(BETA_3_FILE_CONTENTS);
    }

    #[test]
    #[serial]
    fn test_export_toolchain_without_forced_overwrite() {
        create_settings_file();
        create_toolchain_info();

        let channel = channel::BETA_3.to_string();
        export(
            ExportCommand {
                name: None,
                channel: Some(channel.clone()),
                force: false,
            },
            &INPUT_YES[..],
        )
        .expect("should succeed");
        assert_channel_name(channel);
        assert_toolchain_info(BETA_3_FILE_CONTENTS);
    }

    #[test]
    #[serial]
    fn test_export_toolchain_without_exists_toolchain_info_with_valid_channel() {
        create_settings_file();
        // case: path not exist with valid channel
        remove_toolchain_info();
        let channel = channel::BETA_3.to_string();
        export(
            ExportCommand {
                name: None,
                channel: Some(channel.clone()),
                force: false,
            },
            &INPUT_NOP[..],
        )
        .expect("should succeed");
        assert_channel_name(channel);
        assert_toolchain_info(BETA_3_FILE_CONTENTS);
    }

    #[test]
    #[serial]
    fn test_export_toolchain_without_exists_toolchain_info_with_invalid_channel() {
        create_settings_file();
        remove_toolchain_info();
        let channel = channel::BETA_3.to_string();
        let channel_input = format!("{}\n", channel);
        export(
            ExportCommand {
                name: None,
                channel: None,
                force: false,
            },
            channel_input.as_bytes(),
        )
        .expect("should succeed");
        assert_channel_name(channel);
        assert_toolchain_info(BETA_3_FILE_CONTENTS);
    }

    #[test]
    #[should_panic(
        expected = "Invalid channel 'my-channel', expected one of <latest-YYYY-MM-DD|nightly-YYYY-MM-DD|beta-1|beta-2|beta-3|beta-4>."
    )]
    #[serial]
    fn test_export_toolchain_with_invalid_channel() {
        create_settings_file();
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
    #[should_panic(
        expected = "Invalid channel 'my-channel', expected one of <latest-YYYY-MM-DD|nightly-YYYY-MM-DD|beta-1|beta-2|beta-3|beta-4>."
    )]
    #[serial]
    fn test_export_toolchain_without_channel() {
        create_settings_file();
        remove_toolchain_info();
        export(
            ExportCommand {
                name: Some(INVALID_CHANNEL.to_string()),
                channel: None,
                force: false,
            },
            &INPUT_INVALID_CHANNEL[..],
        )
        .unwrap();
    }
}
