use anyhow::{anyhow, bail, Result};
use clap::Parser;
use regex::Regex;
use std::collections::HashSet;
use std::fmt::Write;
use std::fs;
use tracing::{error, info};

use crate::download::{
    download_file_and_unpack, fuelup_bin_dir, unpack_extracted_bins, DownloadCfg,
};

#[derive(Debug, Parser)]
#[clap(override_usage = "\
    fuelup install <COMPONENT>[@<VERSION>] ...")]
pub struct InstallCommand {
    /// Reference to a forc component to add as a dependency
    ///
    /// You can reference components by:{n}
    /// - `<name>`, like `fuelup install forc` (latest version will be used){n}
    /// - `<name>@<version>`, like `cargo add forc@0.14.5`{n}
    #[clap(multiple_values = true, takes_value = true)]
    components: Vec<String>,
}

pub const POSSIBLE_COMPONENTS: [&str; 3] = ["forc", "fuel-core", "fuelup"];

pub fn install_one(name: &str, version: Option<String>) -> Result<DownloadCfg> {
    let fuelup_bin_dir = fuelup_bin_dir();
    if !fuelup_bin_dir.is_dir() {
        fs::create_dir_all(&fuelup_bin_dir).expect("Unable to create fuelup directory");
    }

    let download_cfg = DownloadCfg::new(name, version).expect("Could not create download config");

    info!("Fetching {} {}", &download_cfg.name, &download_cfg.version);

    if download_file_and_unpack(&download_cfg).is_err() {
        bail!("{} {}", &download_cfg.name, &download_cfg.version)
    };

    if unpack_extracted_bins(&fuelup_bin_dir).is_err() {
        bail!("{} {}", &download_cfg.name, &download_cfg.version)
    };

    Ok(download_cfg)
}

pub fn install_all() -> Result<()> {
    for component in ["forc", "fuel-core"].iter() {
        install_one(component, None)?;
    }

    Ok(())
}

pub fn parse_component(component: &str) -> Result<(String, Option<String>)> {
    if component.contains('@') {
        let filtered = component.split('@').collect::<Vec<&str>>();
        let re = Regex::new(r"^\d+\.\d+\.\d+$").unwrap();

        if filtered.len() != 2 {
            bail!("Invalid format for installing component with version: {}. Installing component with version must be in the format <name>@<version> eg. forc@0.14.5", component);
        }

        let name = filtered[0];
        let version = filtered[1];

        if !re.is_match(version) {
            bail!("Invalid format for version: {}. Version must be in the format <major>.<minor>.<patch>", version);
        }

        return Ok((name.to_string(), Some(version.to_string())));
    } else {
        return Ok((component.to_string(), None));
    };
}

pub fn exec(command: InstallCommand) -> Result<()> {
    let InstallCommand { components } = command;

    let mut errored_bins = String::new();
    let mut installed_bins = String::new();
    let mut download_msg = String::new();
    let mut to_download: Vec<DownloadCfg> = Vec::new();

    for component in components.iter() {
        let (name, version) = parse_component(component)?;
        let download_cfg = DownloadCfg::new(&name, version)?;
        to_download.push(download_cfg);
    }

    if components.is_empty() {
        for name in POSSIBLE_COMPONENTS.iter() {
            write!(download_msg, "{} ", name)?;
        }
        install_all()?
    } else {
        let mut waiting_to_download = HashSet::new();

        let temp_components = vec!["forc".to_string(), "fuel".to_string()];
        for name in temp_components.iter() {
            if !waiting_to_download.contains(&name) && POSSIBLE_COMPONENTS.contains(&name.as_str())
            {
                to_download.push(name);
                write!(download_msg, "{} ", name)?;
            }
        }

        info!("Downloading: {}", download_msg);
        for name in to_download.iter() {
            if !waiting_to_download.contains(name) && POSSIBLE_COMPONENTS.contains(&name.as_str()) {
                waiting_to_download.insert(name);
                match install_one(name, None) {
                    Ok(cfg) => writeln!(installed_bins, "- {} {}", cfg.name, cfg.version)?,
                    Err(e) => writeln!(errored_bins, "- {}", e)?,
                };
            }
        }
    }

    if errored_bins.is_empty() {
        info!("\nInstalled:\n{}", installed_bins);
        info!("nThe Fuel toolchain is installed and up to date");
    } else if installed_bins.is_empty() {
        error!(
            "\nfuelup failed to install:\n{}\nYou might need to run `fuelup install` again.",
            errored_bins
        )
    } else {
        info!(
            "\nThe Fuel toolchain is partially installed.\nfuelup failed to install: {}\n\nYou might need to run `fuelup install` again.",
            errored_bins
        );
    };

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_component() {
        assert_eq!(("forc".to_string(), None), parse_component("forc").unwrap());
        assert_eq!(
            ("forc".to_string(), Some("0.14.5".to_string())),
            parse_component("forc@0.14.5").unwrap()
        );

    }

    #[test]
    fn test_parse_component_invalid_component() {
        let invalid_component_msg = |c: &str| {
            format!("Invalid format for installing component with version: {}. Installing component with version must be in the format <name>@<version> eg. forc@0.14.5", c)
        };

        assert_eq!(
            invalid_component_msg("forc@0.1@fuel-core@0.8"),
            parse_component("forc@0.1@fuel-core@0.8").unwrap_err().to_string()
        );
    }

    #[test]
    fn test_parse_component_invalid_version() {
        let invalid_version_msg = |v: &str| format!(
            "Invalid format for version: {}. Version must be in the format <major>.<minor>.<patch>",
           v 
        );
        assert_eq!(
            invalid_version_msg("14"),
            parse_component("forc@14").unwrap_err().to_string()
        );
        assert_eq!(invalid_version_msg("14.0"), parse_component("forc@14.0").unwrap_err().to_string());
        assert_eq!(invalid_version_msg(".14.5"), parse_component("forc@.14.5").unwrap_err().to_string());
        assert_eq!(invalid_version_msg(".14."), parse_component("forc@.14.").unwrap_err().to_string());
    }
}
