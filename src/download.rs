use anyhow::{bail, Result};
use flate2::read::GzDecoder;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tar::Archive;
use tracing::{error, info};

use crate::constants::{
    FUELUP_RELEASE_DOWNLOAD_URL, FUELUP_REPO, FUEL_CORE_RELEASE_DOWNLOAD_URL, FUEL_CORE_REPO,
    GITHUB_API_REPOS_BASE_URL, RELEASES_LATEST, SWAY_RELEASE_DOWNLOAD_URL, SWAY_REPO,
};
use crate::file::hard_or_symlink_file;
use crate::path::fuelup_bin_dir;

#[derive(Debug, Serialize, Deserialize)]
struct LatestReleaseApiResponse {
    url: String,
    tag_name: String,
    name: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct DownloadCfg {
    pub name: String,
    pub version: Version,
    release_url: String,
}

pub mod component {
    pub const FORC: &str = "forc";
    pub const FUEL_CORE: &str = "fuel-core";
    pub const FUELUP: &str = "fuelup";
}

impl DownloadCfg {
    pub fn new(name: &str, version: Option<Version>) -> Result<DownloadCfg> {
        Ok(Self {
            name: name.to_string(),
            version: match version {
                Some(version) => version,
                None => {
                    let latest_tag_url = match name {
                        component::FORC => format!(
                            "{}{}/{}",
                            GITHUB_API_REPOS_BASE_URL, SWAY_REPO, RELEASES_LATEST
                        ),
                        component::FUEL_CORE => format!(
                            "{}{}/{}",
                            GITHUB_API_REPOS_BASE_URL, FUEL_CORE_REPO, RELEASES_LATEST
                        ),
                        component::FUELUP => format!(
                            "{}{}/{}",
                            GITHUB_API_REPOS_BASE_URL, FUELUP_REPO, RELEASES_LATEST
                        ),
                        _ => bail!("Unrecognized component: {}", name),
                    };
                    if let Ok(result) = get_latest_tag(&latest_tag_url) {
                        result
                    } else {
                        bail!("Error getting latest tag for component: {}", name);
                    }
                }
            },
            release_url: match name {
                component::FORC => SWAY_RELEASE_DOWNLOAD_URL.to_string(),
                component::FUEL_CORE => FUEL_CORE_RELEASE_DOWNLOAD_URL.to_string(),
                component::FUELUP => FUELUP_RELEASE_DOWNLOAD_URL.to_string(),
                _ => bail!("Unrecognized component: {}", name),
            },
        })
    }
}

pub fn tarball_name(download_cfg: &DownloadCfg) -> Result<String> {
    match download_cfg.name.as_ref() {
        component::FORC => {
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

        component::FUEL_CORE => {
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
                "fuel-core-{}-{}-{}-{}.tar.gz",
                &download_cfg.version.to_string(),
                architecture,
                vendor,
                os
            ))
        }
        component::FUELUP => {
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
                &download_cfg.version.to_string(),
                architecture,
                vendor,
                os
            ))
        }
        _ => bail!("Unrecognized component: {}", download_cfg.name),
    }
}

pub fn get_latest_tag(github_api_url: &str) -> Result<Version> {
    let handle = ureq::builder().user_agent("fuelup").build();
    let resp = handle.get(github_api_url).call()?;

    let mut data = Vec::new();
    resp.into_reader().read_to_end(&mut data)?;

    let response: LatestReleaseApiResponse = serde_json::from_str(&String::from_utf8_lossy(&data))?;

    // Given a semver version with preceding 'v' (e.g. `v1.2.3`), take the slice after 'v' (e.g. `1.2.3`).
    let version_str = &response.tag_name["v".len()..];
    let version = Version::parse(version_str)?;

    Ok(version)
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

pub fn download_file_and_unpack(download_cfg: &DownloadCfg, dst_dir_path: &Path) -> Result<()> {
    let tarball_name = tarball_name(download_cfg)?;
    let tarball_url = format!(
        "{}/v{}/{}",
        &download_cfg.release_url, &download_cfg.version, &tarball_name
    );

    info!("Fetching binary from {}", &tarball_url);

    let tarball_path = dst_dir_path.join(tarball_name);

    if download_file(&tarball_url, &tarball_path).is_err() {
        error!("Failed to download from {}", &tarball_url,);
    };

    unpack(&tarball_path, dst_dir_path)?;

    Ok(())
}

pub fn unpack_extracted_bins(dir: &PathBuf) -> Result<()> {
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
                if fs::copy(&bin_file.path(), dir.join(&bin_file.file_name())).is_ok() {
                    let fuelup_bin_path = fuelup_bin_dir().join("fuelup");
                    let bin_path = dir.join(bin_file.file_name());

                    hard_or_symlink_file(&fuelup_bin_path, &bin_path)?;
                };
            }

            fs::remove_dir_all(sub_path)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile;

    pub(crate) fn with_toolchain_dir<F>(f: F) -> Result<()>
    where
        F: FnOnce(tempfile::TempDir) -> Result<()>,
    {
        let toolchain_bin_dir = tempfile::tempdir()?;
        f(toolchain_bin_dir)
    }

    #[test]
    fn test_unpack_extracted_bins() -> Result<()> {
        with_toolchain_dir(|dir| {
            let mock_bin_dir = dir.path().join("bin");
            let extracted_bins_dir = mock_bin_dir.join("forc-binaries");
            fs::create_dir(&mock_bin_dir).unwrap();
            fs::create_dir(&extracted_bins_dir).unwrap();

            let mock_bin_file_1 = extracted_bins_dir.join("forc-mock-exec-1");
            let mock_bin_file_2 = extracted_bins_dir.join("forc-mock-exec-2");

            fs::File::create(mock_bin_file_1).unwrap();
            fs::File::create(mock_bin_file_2).unwrap();

            assert!(extracted_bins_dir.exists());
            assert!(!dir.path().join("forc-mock-exec-1").to_owned().exists());
            assert!(!dir.path().join("forc-mock-exec-2").to_owned().exists());

            unpack_extracted_bins(&mock_bin_dir.to_path_buf()).unwrap();

            assert!(!extracted_bins_dir.exists());
            assert!(mock_bin_dir.join("forc-mock-exec-1").to_owned().exists());
            assert!(mock_bin_dir.join("forc-mock-exec-2").to_owned().exists());
            Ok(())
        })
    }
}
