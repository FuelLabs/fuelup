use anyhow::{bail, Result};
use dirs::home_dir;
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tar::Archive;
use tracing::{error, info};

use crate::constants::FUELUP_DIR;

#[derive(Debug, Serialize, Deserialize)]
struct LatestReleaseApiResponse {
    url: String,
    tag_name: String,
    name: String,
}

pub fn forc_bin_tarball_name() -> Result<String> {
    let os = match std::env::consts::OS {
        "macos" => "darwin",
        "linux" => "linux",
        unsupported_os => bail!("Unsupported os: {}", unsupported_os),
    };
    let architecture = match std::env::consts::ARCH {
        "aarch64" => "arm64",
        "x86_64" => "amd64",
        unsupported_arch => bail!("Unsupported architecture: {}", unsupported_arch),
    };

    Ok(format!("forc-binaries-{}_{}.tar.gz", os, architecture))
}

pub fn fuel_core_bin_tarball_name(version: &str) -> Result<String> {
    let architecture = match std::env::consts::ARCH {
        "aarch64" => "aarch64",
        "x86_64" => "x86_64",
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
        "fuel-core-{}-{}-{}-{}.tar.gz",
        // strip the 'v' from the version string to match the file name of the releases
        &version[1..version.len()],
        architecture,
        vendor,
        os
    ))
}

pub fn get_latest_tag(github_api_url: &str) -> Result<String> {
    let handle = ureq::builder().user_agent("fuelup").build();
    let resp = handle.get(github_api_url).call()?;

    let mut data = Vec::new();
    resp.into_reader().read_to_end(&mut data)?;

    let response: LatestReleaseApiResponse = serde_json::from_str(&String::from_utf8_lossy(&data))?;
    Ok(response.tag_name)
}

pub fn fuelup_bin_dir() -> PathBuf {
    home_dir().unwrap().join(FUELUP_DIR).join("bin")
}

fn unpack(tar_path: &Path, dst: &Path) -> Result<()> {
    let tar_gz = File::open(tar_path)?;
    let decompressed = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(decompressed);

    if let Err(e) = archive.unpack(dst) {
        error!(
            "{}. The archive could be corrupted or the release may not be ready yet",
            e
        );
    };

    Ok(())
}

pub fn download_file(url: &str, path: &PathBuf) -> Result<File> {
    let handle = ureq::builder().user_agent("fuelup").build();
    let resp = handle.get(url).call()?;

    let mut data = Vec::new();
    resp.into_reader().read_to_end(&mut data)?;

    let mut file = OpenOptions::new().write(true).create(true).open(&path)?;
    if let Err(e) = file.write_all(&data) {
        error!("Something went wrong writing to {}: {}", path.display(), e)
    };

    Ok(file)
}

pub fn download_file_and_unpack(
    github_release_url: &str,
    tag: &str,
    tarball_name: &str,
) -> Result<()> {
    let tarball_url = format!("{}/{}/{}", &github_release_url, &tag, &tarball_name);

    info!("Fetching binary from {}", &tarball_url);

    let fuelup_bin_dir = fuelup_bin_dir();

    let tarball_path = fuelup_bin_dir.join(tarball_name);

    if download_file(&tarball_url, &tarball_path).is_err() {
        error!(
            "Failed to download from {} and write to path {}",
            &tarball_url,
            &tarball_path.display()
        );
    };

    unpack(&tarball_path, &fuelup_bin_dir)?;

    fs::remove_file(&tarball_path)?;

    Ok(())
}
