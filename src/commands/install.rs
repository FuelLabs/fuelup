use std::fs;

use anyhow::{bail, Result};
use clap::Parser;
use tracing::info;

use crate::constants::{FUEL_CORE_RELEASE_DOWNLOAD_URL, SWAY_RELEASE_DOWNLOAD_URL};
use crate::download::{
    download_file_and_unpack, fuelup_bin_dir, unpack_extracted_bins, DownloadCfg,
};
use crate::{
    constants::{FUEL_CORE_REPO, GITHUB_API_REPOS_BASE_URL, RELEASES_LATEST, SWAY_REPO},
    download::get_latest_tag,
};

#[derive(Debug, Parser)]
pub struct InstallCommand {}

pub fn install_component(name: &str, version: Option<String>) -> Result<()> {
    info!("\nDownloading the Fuel toolchain\n");

    let fuelup_bin_dir = fuelup_bin_dir();
    if !fuelup_bin_dir.is_dir() {
        fs::create_dir_all(&fuelup_bin_dir)?;
    }

    let download_cfg = DownloadCfg::new(name, version)?;

    info!("Fetching {} {}", &download_cfg.name, &download_cfg.version);
    download_file_and_unpack(&download_cfg)?;

    unpack_extracted_bins(&fuelup_bin_dir)?;

    Ok(())
}

pub fn install() -> Result<()> {
    info!("\nDownloading the Fuel toolchain\n");

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

    unpack_extracted_bins(&fuelup_bin_dir)?;

    info!(
        "\n\nInstalled: forc {}, fuel-core {}",
        forc_release_latest_tag, fuel_core_release_latest_tag
    );
    info!("\nThe Fuel toolchain is installed now. Great!");
    info!(
        "\nYou might need to add {} to your path.",
        fuelup_bin_dir.display()
    );

    Ok(())
}
