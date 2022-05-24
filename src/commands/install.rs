use std::fs;
use std::path::Path;

use anyhow::Result;
use clap::Parser;
use dirs::home_dir;
use tracing::info;

use crate::constants::{FUELUP_PATH, FUEL_CORE_RELEASE_DOWNLOAD_URL, SWAY_RELEASE_DOWNLOAD_URL};
use crate::download::download_file_and_unpack;
use crate::{
    constants::{FUEL_CORE_REPO, GITHUB_API_REPOS_BASE_URL, RELEASES_LATEST, SWAY_REPO},
    download::{forc_bin_tarball_name, fuel_core_bin_tarball_name, get_latest_tag},
};

#[derive(Debug, Parser)]
pub struct InstallCommand {}

pub fn install() -> Result<()> {
    info!("Downloading the Forc toolchain");

    let forc_release_latest_tag = get_latest_tag(&format!(
        "{}{}/{}",
        GITHUB_API_REPOS_BASE_URL, SWAY_REPO, RELEASES_LATEST
    ))?;
    let fuel_core_release_latest_tag = get_latest_tag(&format!(
        "{}{}/{}",
        GITHUB_API_REPOS_BASE_URL, FUEL_CORE_REPO, RELEASES_LATEST
    ))?;

    let forc_bin_tarball_name = forc_bin_tarball_name()?;
    let fuel_core_bin_tarball_name = fuel_core_bin_tarball_name(&fuel_core_release_latest_tag)?;

    download_file_and_unpack(
        SWAY_RELEASE_DOWNLOAD_URL,
        &forc_release_latest_tag,
        &forc_bin_tarball_name,
    )?;
    download_file_and_unpack(
        FUEL_CORE_RELEASE_DOWNLOAD_URL,
        &fuel_core_release_latest_tag,
        &fuel_core_bin_tarball_name,
    )?;

    let fuelup_path = home_dir().unwrap().join(Path::new(FUELUP_PATH));
    for entry in std::fs::read_dir(&fuelup_path)? {
        let sub_path = entry?.path();

        if sub_path.is_dir() {
            for bin in std::fs::read_dir(&sub_path)? {
                let bin_file = bin?;
                info!(
                    "Unpacking and moving {} to {}/.fuelup...",
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

    Ok(())
}
