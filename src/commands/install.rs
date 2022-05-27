use std::fs;

use anyhow::{bail, Result};
use clap::Parser;
use tracing::{error, info};

use crate::constants::{FUEL_CORE_RELEASE_DOWNLOAD_URL, SWAY_RELEASE_DOWNLOAD_URL};
use crate::download::{download_file_and_unpack, fuelup_bin_dir};
use crate::{
    constants::{FUEL_CORE_REPO, GITHUB_API_REPOS_BASE_URL, RELEASES_LATEST, SWAY_REPO},
    download::{forc_bin_tarball_name, fuel_core_bin_tarball_name, get_latest_tag},
};

#[derive(Debug, Parser)]
pub struct InstallCommand {}

pub fn install() -> Result<()> {
    info!("\nDownloading the Forc toolchain\n");

    let forc_release_latest_tag = match get_latest_tag(&format!(
        "{}{}/{}",
        GITHUB_API_REPOS_BASE_URL, SWAY_REPO, RELEASES_LATEST
    )) {
        Ok(t) => t,
        Err(_) => bail!("Failed to fetch latest forc release tag from GitHub API"),
    };

    let fuel_core_release_latest_tag = match get_latest_tag(&format!(
        "{}{}/{}",
        GITHUB_API_REPOS_BASE_URL, FUEL_CORE_REPO, RELEASES_LATEST
    )) {
        Ok(t) => t,
        Err(_) => bail!("Failed to fetch latest fuel-core release tag from GitHub API"),
    };

    let fuelup_bin_dir = fuelup_bin_dir();
    if !fuelup_bin_dir.is_dir() {
        fs::create_dir_all(&fuelup_bin_dir)?;
    }

    let forc_bin_tarball_name = forc_bin_tarball_name()?;
    let fuel_core_bin_tarball_name = fuel_core_bin_tarball_name(&fuel_core_release_latest_tag)?;

    info!("Fetching forc {}", &forc_release_latest_tag);

    let mut installed_bins_message = String::new();
    let mut errored_bins_message = String::new();

    match download_file_and_unpack(
        SWAY_RELEASE_DOWNLOAD_URL,
        &forc_release_latest_tag,
        &forc_bin_tarball_name,
    ) {
        Ok(()) => installed_bins_message.push_str(&format!("forc {}", &forc_release_latest_tag)),
        Err(_) => errored_bins_message.push_str(&format!("forc {}", &forc_release_latest_tag)),
    };

    info!("Fetching fuel-core {}", &fuel_core_release_latest_tag);
    match download_file_and_unpack(
        FUEL_CORE_RELEASE_DOWNLOAD_URL,
        &fuel_core_release_latest_tag,
        &fuel_core_bin_tarball_name,
    ) {
        Ok(()) => {
            if !installed_bins_message.is_empty() {
                installed_bins_message.push_str(&", ")
            }
            installed_bins_message.push_str(&format!("fuel-core {}", &fuel_core_release_latest_tag))
        }
        Err(_) => {
            if !errored_bins_message.is_empty() {
                errored_bins_message.push_str(&", ")
            }
            errored_bins_message.push_str(&format!("fuel-core {}", &fuel_core_release_latest_tag))
        }
    };

    for entry in std::fs::read_dir(&fuelup_bin_dir)? {
        let sub_path = entry?.path();

        if sub_path.is_dir() {
            for bin in std::fs::read_dir(&sub_path)? {
                let bin_file = bin?;
                info!(
                    "Unpacking and moving {} to {}",
                    &bin_file.file_name().to_string_lossy(),
                    fuelup_bin_dir.display()
                );
                fs::copy(&bin_file.path(), fuelup_bin_dir.join(&bin_file.file_name()))?;
            }

            fs::remove_dir_all(sub_path)?;
        }
    }

    info!("\nInstalled: {}\n", installed_bins_message);

    if errored_bins_message.is_empty() {
        info!("The Forc toolchain is installed now. Great!");
    } else if installed_bins_message.is_empty() {
        error!("fuelup failed to install: {}", errored_bins_message)
    } else {
        info!(
            "The Forc toolchain is partially installed.\nfuelup failed to install: {}",
            errored_bins_message
        );
    };
    info!(
        "\nYou might need to add {} to your path.",
        fuelup_bin_dir.display()
    );

    Ok(())
}
