use anyhow::{anyhow, bail, Result};
use component;
use flate2::read::GzDecoder;
use semver::Version;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use sha2::Sha256;
use std::env;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Duration;
use std::{fs, thread};
use tar::Archive;
use tempfile::tempdir_in;
use tracing::warn;
use tracing::{error, info};

use crate::channel::Channel;
use crate::channel::Package;
use crate::constants::{
    CHANNEL_LATEST_URL, FUELUP_RELEASE_DOWNLOAD_URL, FUEL_CORE_RELEASE_DOWNLOAD_URL,
    SWAY_RELEASE_DOWNLOAD_URL,
};
use crate::file::hard_or_symlink_file;
use crate::path::fuelup_bin;
use crate::path::fuelup_dir;
use crate::target_triple::TargetTriple;
use crate::toolchain::OfficialToolchainDescription;

#[derive(Debug, Serialize, Deserialize)]
struct LatestReleaseApiResponse {
    url: String,
    tag_name: String,
    name: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct DownloadCfg {
    pub name: String,
    pub target: TargetTriple,
    pub version: Version,
    tarball_name: String,
    tarball_url: String,
    hash: Option<String>,
}

impl DownloadCfg {
    pub fn new(name: &str, target: TargetTriple, version: Option<Version>) -> Result<Self> {
        let version = match version {
            Some(version) => version,
            None => get_latest_version(name).map_err(|e| {
                anyhow!("Error getting latest tag for component: {:?}: {}", name, e)
            })?,
        };

        let release_url = match name {
            component::FORC => SWAY_RELEASE_DOWNLOAD_URL.to_string(),
            component::FUEL_CORE => FUEL_CORE_RELEASE_DOWNLOAD_URL.to_string(),
            component::FUELUP => FUELUP_RELEASE_DOWNLOAD_URL.to_string(),
            _ => bail!("Unrecognized component: {}", name),
        };
        let tarball_name = tarball_name(name, &version, &target)?;
        let tarball_url = format!("{}/v{}/{}", &release_url, &version, &tarball_name);

        Ok(Self {
            name: name.to_string(),
            target,
            version,
            tarball_name,
            tarball_url,
            hash: None,
        })
    }

    pub fn from_package(name: &str, package: Package) -> Result<Self> {
        let target = TargetTriple::from_component(name)?;
        let tarball_name = tarball_name(name, &package.version, &target)?;
        let tarball_url = package.target[&target.to_string()].url.clone();
        let hash = Some(package.target[&target.to_string()].hash.clone());
        Ok(Self {
            name: name.to_string(),
            target,
            version: package.version,
            tarball_name,
            tarball_url,
            hash,
        })
    }
}

pub fn tarball_name(name: &str, version: &Version, target: &TargetTriple) -> Result<String> {
    match name {
        component::FORC => Ok(format!("forc-binaries-{}.tar.gz", target)),
        component::FUEL_CORE => Ok(format!("fuel-core-{}-{}.tar.gz", version, target)),
        component::FUELUP => Ok(format!("fuelup-{}-{}.tar.gz", version, target)),
        _ => bail!("Unrecognized component: {}", name),
    }
}

pub fn get_latest_version(name: &str) -> Result<Version> {
    let handle = ureq::builder().user_agent("fuelup").build();
    let mut data = Vec::new();
    if name == component::FUELUP {
        const FUELUP_RELEASES_API_URL: &str =
            "https://api.github.com/repos/FuelLabs/fuelup/releases/latest";
        let resp = handle.get(FUELUP_RELEASES_API_URL).call()?;
        resp.into_reader().read_to_end(&mut data)?;
        let response: LatestReleaseApiResponse =
            serde_json::from_str(&String::from_utf8_lossy(&data))?;

        let version_str = &response.tag_name["v".len()..];
        let version = Version::parse(version_str)?;
        Ok(version)
    } else {
        let resp = handle.get(CHANNEL_LATEST_URL).call()?;

        resp.into_reader().read_to_end(&mut data)?;
        let tmp_dir = tempdir_in(&fuelup_dir())?;

        if let Ok(channel) = Channel::from_dist_channel(
            &OfficialToolchainDescription::from_str("latest")?,
            tmp_dir.into_path(),
        ) {
            Ok(channel.pkg[name].version.clone())
        } else {
            bail!("Failed to get 'latest' channel")
        }
    }
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

    fs::remove_file(&tar_path)?;
    Ok(())
}

pub fn download_file(url: &str, path: &PathBuf, hasher: &mut Sha256) -> Result<()> {
    const RETRY_ATTEMPTS: u8 = 4;
    const RETRY_DELAY_SECS: u64 = 3;

    // auto detect http proxy setting.
    let handle = if let Ok(proxy) = env::var("http_proxy") {
        ureq::builder()
            .user_agent("fuelup")
            .proxy(ureq::Proxy::new(proxy)?)
            .build()
    } else {
        ureq::builder().user_agent("fuelup").build()
    };

    let mut file = OpenOptions::new().write(true).create(true).open(&path)?;

    for _ in 1..RETRY_ATTEMPTS {
        match handle.get(url).call() {
            Ok(response) => {
                let mut data = Vec::new();
                response.into_reader().read_to_end(&mut data)?;

                if let Err(e) = file.write_all(&data) {
                    error!(
                        "Something went wrong writing data to {}: {}",
                        path.display(),
                        e
                    )
                };

                hasher.update(data);
                return Ok(());
            }
            Err(ureq::Error::Status(404, r)) => {
                // We've reached download_file stage, which means the tag must be correct.
                error!("Failed to download from {}", &url);
                let retry: Option<u64> = r.header("retry-after").and_then(|h| h.parse().ok());
                let retry = retry.unwrap_or(RETRY_DELAY_SECS);
                info!("Retrying..");
                thread::sleep(Duration::from_secs(retry));
            }
            Err(e) => {
                // handle other status code and non-status code errors
                bail!("Unexpected error: {}", e.to_string());
            }
        }
    }

    bail!("Could not download file");
}

pub fn download_file_and_unpack(download_cfg: &DownloadCfg, dst_dir_path: &Path) -> Result<()> {
    info!("Fetching binary from {}", &download_cfg.tarball_url);
    if download_cfg.hash.is_none() {
        warn!(
            "Downloading component {} without verifying checksum",
            &download_cfg.name
        );
    }

    let tarball_path = dst_dir_path.join(&download_cfg.tarball_name);

    let mut hasher = Sha256::new();
    if download_file(&download_cfg.tarball_url, &tarball_path, &mut hasher).is_err() {
        bail!(
            "Failed to download {} - the release may not exist or may not be ready yet.",
            &download_cfg.tarball_name
        );
    };

    let actual_hash = format!("{:x}", hasher.finalize());
    if download_cfg.hash.is_some() && (&actual_hash != download_cfg.hash.as_ref().unwrap()) {
        bail!(
            "Attempt to verify sha256 checksum failed:\ndownloaded file: {}\npublished sha256 hash: {}",
            &actual_hash,
            download_cfg.hash.as_ref().unwrap()
        )
    }

    unpack(&tarball_path, dst_dir_path)?;

    Ok(())
}

pub fn link_to_fuelup(bins: Vec<PathBuf>) -> Result<()> {
    let fuelup_bin_path = fuelup_bin();
    for path in bins {
        hard_or_symlink_file(&fuelup_bin_path, &path)?;
    }
    Ok(())
}

pub fn unpack_bins(dir: &Path, dst_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut downloaded: Vec<PathBuf> = Vec::new();
    for entry in std::fs::read_dir(&dir)? {
        let sub_path = entry?.path();

        if sub_path.is_dir() {
            for bin in std::fs::read_dir(&sub_path)? {
                let bin_file = bin?;
                let bin_file_name = bin_file.file_name();
                info!(
                    "Unpacking and moving {} to {}",
                    &bin_file_name.to_string_lossy(),
                    dir.display()
                );
                fs::copy(bin_file.path(), dir.join(&bin_file_name))?;
                downloaded.push(dst_dir.join(bin_file_name));
            }

            fs::remove_dir_all(sub_path)?;
        }
    }

    Ok(downloaded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use dirs::home_dir;
    use tempfile;

    pub(crate) fn with_toolchain_dir<F>(f: F) -> Result<()>
    where
        F: FnOnce(tempfile::TempDir) -> Result<()>,
    {
        let toolchain_bin_dir = tempfile::tempdir()?;
        f(toolchain_bin_dir)
    }

    #[test]
    fn test_unpack_and_link_bins() -> Result<()> {
        with_toolchain_dir(|dir| {
            let mock_bin_dir = tempfile::tempdir_in(&dir).unwrap().into_path();
            let extracted_bins_dir = mock_bin_dir.join("forc-binaries");
            let mock_fuelup_dir = tempfile::tempdir_in(home_dir().unwrap()).unwrap();
            let _mock_fuelup_bin_dir = tempfile::tempdir_in(&mock_fuelup_dir).unwrap();
            fs::create_dir(&extracted_bins_dir).unwrap();

            let mock_bin_file_1 = extracted_bins_dir.join("forc-mock-exec-1");
            let mock_bin_file_2 = extracted_bins_dir.join("forc-mock-exec-2");

            fs::File::create(mock_bin_file_1).unwrap();
            fs::File::create(mock_bin_file_2).unwrap();

            assert!(extracted_bins_dir.exists());
            assert!(dir.path().join("forc-mock-exec-1").metadata().is_err());
            assert!(dir.path().join("forc-mock-exec-2").metadata().is_err());

            unpack_bins(&mock_bin_dir, &mock_fuelup_dir.into_path()).unwrap();

            assert!(!extracted_bins_dir.exists());
            assert!(mock_bin_dir.join("forc-mock-exec-1").metadata().is_ok());
            assert!(mock_bin_dir.join("forc-mock-exec-2").metadata().is_ok());
            Ok(())
        })
    }
}
