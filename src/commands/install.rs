use anyhow::{bail, Result};
use clap::Parser;
use semver::Version;
use std::fmt::Write;
use std::fs;
use tracing::{error, info};

use crate::download::{
    component, download_file_and_unpack, fuelup_bin_dir, unpack_extracted_bins, DownloadCfg,
};

#[derive(Debug, Parser)]
#[clap(override_usage = "\
    fuelup install <COMPONENT>[@<VERSION>] ...")]
pub struct InstallCommand {
    /// Reference to a forc component to add as a dependency
    ///
    /// You can reference components by:{n}
    /// - `<name>`, like `fuelup install forc` (latest version will be used){n}
    /// - `<name>@<version>`, like `fuelup install forc@0.14.5`{n}
    #[clap(multiple_values = true)]
    components: Vec<String>,
}

pub fn install_one(download_cfg: DownloadCfg) -> Result<DownloadCfg> {
    let fuelup_bin_dir = fuelup_bin_dir();
    if !fuelup_bin_dir.is_dir() {
        fs::create_dir_all(&fuelup_bin_dir).expect("Unable to create fuelup directory");
    }

    info!("Fetching {} {}", &download_cfg.name, &download_cfg.version);

    if download_file_and_unpack(&download_cfg).is_err() {
        bail!("{} {}", &download_cfg.name, &download_cfg.version)
    };

    if unpack_extracted_bins(&fuelup_bin_dir).is_err() {
        bail!("{} {}", &download_cfg.name, &download_cfg.version)
    };

    Ok(download_cfg)
}

pub fn parse_component(component: &str) -> Result<(String, Option<String>)> {
    if component.contains('@') {
        let split = component.split('@').collect::<Vec<&str>>();

        if split.len() != 2 {
            bail!("Invalid format for installing component with version: {}. Installing component with version must be in the format <name>@<version> eg. forc@0.14.5", component);
        }

        let name = split[0];
        let mut version = split[1];

        if version.starts_with('v') {
            version = &version[1..version.len()]
        }

        if let Err(e) = Version::parse(version) {
            bail!(
                "Error parsing version {} - {}. Version input must be in the format <major>.<minor>.<patch>",
                version, e
            )
        };

        Ok((name.to_string(), Some(version.to_string())))
    } else {
        Ok((component.to_string(), None))
    }
}

pub fn exec(command: InstallCommand) -> Result<()> {
    let InstallCommand { components } = command;

    let mut errored_bins = String::new();
    let mut installed_bins = String::new();
    let mut download_msg = String::new();

    if components.is_empty() {
        let mut cfgs: Vec<DownloadCfg> = Vec::new();

        for component in [component::FORC, component::FUEL_CORE, component::FUELUP].iter() {
            write!(download_msg, "{} ", component)?;
            let download_cfg: DownloadCfg = DownloadCfg::new(component, None)?;
            cfgs.push(download_cfg);
        }

        info!("Downloading: {}", download_msg);
        for cfg in cfgs {
            match install_one(cfg) {
                Ok(cfg) => writeln!(installed_bins, "- {} {}", cfg.name, cfg.version)?,
                Err(e) => writeln!(errored_bins, "- {}", e)?,
            };
        }
    } else {
        let mut to_download: Vec<DownloadCfg> = Vec::new();

        for component in components.iter() {
            let (name, version) = parse_component(component)?;
            let download_cfg = DownloadCfg::new(&name, version)?;

            if to_download
                .iter()
                .map(|t| t.name.clone())
                .collect::<String>()
                .contains(&download_cfg.name)
            {
                bail!(
                    "Invalid command due to duplicate input: {}",
                    &download_cfg.name
                );
            }

            to_download.push(download_cfg);
            write!(download_msg, "{} ", name)?;
        }

        info!("Downloading: {}", download_msg);
        for cfg in to_download {
            match install_one(cfg) {
                Ok(cfg) => writeln!(installed_bins, "- {} {}", cfg.name, cfg.version)?,
                Err(e) => writeln!(errored_bins, "- {}", e)?,
            };
        }
    }

    if errored_bins.is_empty() {
        info!("\nInstalled:\n{}", installed_bins);
        info!("\nThe Fuel toolchain is installed and up to date");
    } else if installed_bins.is_empty() {
        error!("\nfuelup failed to install:\n{}", errored_bins)
    } else {
        info!(
            "\nThe Fuel toolchain is partially installed.\nfuelup failed to install: {}",
            errored_bins
        );
    };

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::download::component;

    use super::*;
    #[test]
    fn test_parse_component() {
        assert_eq!(
            (component::FORC.to_string(), None),
            parse_component(component::FORC).unwrap()
        );
        assert_eq!(
            (component::FORC.to_string(), Some("0.14.5".to_string())),
            parse_component("forc@0.14.5").unwrap()
        );
        assert_eq!(
            (component::FORC.to_string(), Some("0.14.5".to_string())),
            parse_component("forc@v0.14.5").unwrap()
        );
    }

    #[test]
    fn test_duplicate_inputs() {
        let invalid_component_msg =
            |c: &str| format!("Invalid command due to duplicate input: {}", c);

        assert_eq!(
            invalid_component_msg(component::FORC),
            exec(InstallCommand {
                components: vec![component::FORC.to_string(), component::FORC.to_string()]
            })
            .unwrap_err()
            .to_string()
        );
    }

    #[test]
    fn test_parse_component_invalid_component() {
        let invalid_component_msg = |c: &str| {
            format!("Invalid format for installing component with version: {}. Installing component with version must be in the format <name>@<version> eg. forc@0.14.5", c)
        };

        assert_eq!(
            invalid_component_msg("forc@0.1@fuel-core@0.8"),
            parse_component("forc@0.1@fuel-core@0.8")
                .unwrap_err()
                .to_string()
        );
    }

    #[test]
    fn test_parse_component_invalid_version() {
        let invalid_version_msg = |version: &str, version_type: &str| {
            format!(
            "Error parsing version {} - unexpected end of input while parsing {} version number. Version input must be in the format <major>.<minor>.<patch>",
            version, version_type
            )
        };

        let unexpected_char_msg = |version: &str| {
            format!(
                "Error parsing version {} - unexpected character '.' while parsing major version number. Version input must be in the format <major>.<minor>.<patch>",
                version
                )
        };
        assert_eq!(
            invalid_version_msg("1", "major"),
            parse_component("forc@1").unwrap_err().to_string()
        );
        assert_eq!(
            invalid_version_msg("1", "major"),
            parse_component("forc@v1").unwrap_err().to_string()
        );
        assert_eq!(
            invalid_version_msg("1.0", "minor"),
            parse_component("forc@1.0").unwrap_err().to_string()
        );
        assert_eq!(
            invalid_version_msg("1.0", "minor"),
            parse_component("forc@v1.0").unwrap_err().to_string()
        );
        assert_eq!(
            invalid_version_msg("1.0.", "patch"),
            parse_component("forc@1.0.").unwrap_err().to_string()
        );
        assert_eq!(
            unexpected_char_msg(".1"),
            parse_component("forc@.1").unwrap_err().to_string()
        );
        assert_eq!(
            unexpected_char_msg(".1."),
            parse_component("forc@.1.").unwrap_err().to_string()
        );
    }
}
