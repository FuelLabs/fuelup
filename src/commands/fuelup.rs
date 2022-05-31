use anyhow::{bail, Result};
use clap::Parser;
use tracing::info;

use crate::{
    constants::{FUELUP_RELEASE_DOWNLOAD_URL, GITHUB_API_REPOS_BASE_URL, RELEASES_LATEST},
    download::{download_file_and_unpack, fuelup_bin_dir, get_latest_tag, unpack_extracted_bins},
};

#[derive(Debug, Parser)]
pub enum FuelupCommand {
    /// Updates fuelup
    Update,
}

#[derive(Debug, Parser)]
struct UpdateCommand {}

pub const FUELUP_VERSION: &str = concat!("v", clap::crate_version!());

fn fuelup_bin_tarball_name(version: &str) -> Result<String> {
    let architecture = match std::env::consts::ARCH {
        "aarch64" | "x86_64" => std::env::consts::ARCH,
        unsupported_arch => bail!("Unsupported architecture: {}", unsupported_arch),
    };

    let vendor = match std::env::consts::OS {
        "macos" => "apple",
        _ => "unknown",
    };

    let os = match std::env::consts::OS {
        "macos" => "darwin",
        "linux" => "linux-gnu",
        unsupported_os => bail!("Unsupported os: {}", unsupported_os),
    };

    Ok(format!(
        "fuelup-{}-{}-{}-{}.tar.gz",
        // strip the 'v' from the version string to match the file name of the releases
        &version[1..version.len()],
        architecture,
        vendor,
        os
    ))
}

pub fn self_update() -> Result<()> {
    let fuelup_release_latest_tag = match get_latest_tag(&format!(
        "{}{}/{}",
        GITHUB_API_REPOS_BASE_URL, "fuelup", RELEASES_LATEST
    )) {
        Ok(t) => t,
        Err(_) => bail!("Failed to fetch latest fuelup release tag from GitHub API"),
    };

    if fuelup_release_latest_tag == FUELUP_VERSION {
        info!("fuelup unchanged - at latest version ({})", FUELUP_VERSION);
        return Ok(());
    }

    let fuelup_bin_tarball_name = fuelup_bin_tarball_name(&fuelup_release_latest_tag)?;

    info!("Fetching fuelup {}", &fuelup_release_latest_tag);
    download_file_and_unpack(
        FUELUP_RELEASE_DOWNLOAD_URL,
        &fuelup_release_latest_tag,
        &fuelup_bin_tarball_name,
    )?;

    unpack_extracted_bins(&fuelup_bin_dir())?;

    Ok(())
}
