use anyhow::{bail, Result};
use clap::Parser;
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
            let download_cfg = DownloadCfg::new(component, None)?;

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
            write!(download_msg, "{} ", component)?;
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
    use super::*;

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
}
