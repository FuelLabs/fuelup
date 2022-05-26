use std::fs;
use std::path::Path;

use anyhow::{bail, Result};
use clap::Parser;
use dirs::home_dir;
use tracing::info;

use crate::constants::{
    FUELUP_BIN_PATH, FUEL_CORE_RELEASE_DOWNLOAD_URL, SWAY_RELEASE_DOWNLOAD_URL,
};
use crate::download::download_file_and_unpack;
use crate::{
    constants::{FUEL_CORE_REPO, GITHUB_API_REPOS_BASE_URL, RELEASES_LATEST, SWAY_REPO},
    download::{forc_bin_tarball_name, fuel_core_bin_tarball_name, get_latest_tag},
};

#[derive(Debug, Parser)]
pub struct InstallCommand {}

pub fn install() -> Result<()> {
    info!("Downloading the Forc toolchain");

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

    let forc_bin_tarball_name = forc_bin_tarball_name()?;
    let fuel_core_bin_tarball_name = fuel_core_bin_tarball_name(&fuel_core_release_latest_tag)?;

    info!("Fetching forc {}", &forc_release_latest_tag);
    download_file_and_unpack(
        SWAY_RELEASE_DOWNLOAD_URL,
        &forc_release_latest_tag,
        &forc_bin_tarball_name,
    )?;

    info!("Fetching fuel-core {}", &fuel_core_release_latest_tag);
    download_file_and_unpack(
        FUEL_CORE_RELEASE_DOWNLOAD_URL,
        &fuel_core_release_latest_tag,
        &fuel_core_bin_tarball_name,
    )?;

    let fuelup_path = home_dir().unwrap().join(Path::new(FUELUP_BIN_PATH));
    for entry in std::fs::read_dir(&fuelup_path)? {
        let sub_path = entry?.path();

        if sub_path.is_dir() {
            for bin in std::fs::read_dir(&sub_path)? {
                let bin_file = bin?;
                info!(
                    "Unpacking and moving {} to {}/.fuelup/bin",
                    &bin_file.file_name().to_string_lossy(),
                    home_dir().unwrap().display()
                );
                fs::copy(
                    &bin_file.path(),
                    Path::new(&fuelup_path).join(&bin_file.file_name()),
                )?;
            }

            fs::remove_dir_all(sub_path)?;
        }
    }

    info!("The Forc toolchain is installed now. Great!\n");
    info!("You might need to add $HOME/.fuelup/bin to your path.");

    Ok(())
}
