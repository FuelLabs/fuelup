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

use crate::constants::{
    FUELUP_DIR, FUELUP_RELEASE_DOWNLOAD_URL, FUEL_CORE_RELEASE_DOWNLOAD_URL, FUEL_CORE_REPO,
    GITHUB_API_REPOS_BASE_URL, RELEASES_LATEST, SWAY_RELEASE_DOWNLOAD_URL, SWAY_REPO,
};

#[derive(Debug, Serialize, Deserialize)]
struct LatestReleaseApiResponse {
    url: String,
    tag_name: String,
    name: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct DownloadCfg {
    pub name: String,
    pub version: String,
    release_url: String,
}

impl DownloadCfg {
    pub fn new(name: &str, version: Option<String>) -> Result<DownloadCfg> {
        Ok(Self {
            name: name.to_string(),
            version: match version {
                Some(version) => {
                    if version.starts_with('v') {
                        version
                    } else {
                        "v".to_string() + &version
                    }
                }
                None => {
                    let latest_tag_url = match name {
                        "forc" => format!(
                            "{}{}/{}",
                            GITHUB_API_REPOS_BASE_URL, SWAY_REPO, RELEASES_LATEST
                        ),
                        "fuel-core" => format!(
                            "{}{}/{}",
                            GITHUB_API_REPOS_BASE_URL, FUEL_CORE_REPO, RELEASES_LATEST
                        ),
                        "fuelup" => format!(
                            "{}{}/{}",
                            GITHUB_API_REPOS_BASE_URL, "fuelup", RELEASES_LATEST
                        ),
                        _ => bail!("Unrecognized component: {}", name),
                    };
                    if let Ok(result) = get_latest_tag(&latest_tag_url) {
                        result
                    } else {
                        bail!("Error getting latest tag");
                    }
                }
            },
            release_url: match name {
                "forc" => SWAY_RELEASE_DOWNLOAD_URL.to_string(),
                "fuel-core" => FUEL_CORE_RELEASE_DOWNLOAD_URL.to_string(),
                "fuelup" => FUELUP_RELEASE_DOWNLOAD_URL.to_string(),
                _ => bail!("Unrecognized component: {}", name),
            },
        })
    }
}

pub fn tarball_name(download_cfg: &DownloadCfg) -> Result<String> {
    match download_cfg.name.as_ref() {
        "forc" => {
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

        "fuel-core" => {
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
                &download_cfg.version[1..download_cfg.version.len()].to_string(),
                architecture,
                vendor,
                os
            ))
        }
        "fuelup" => {
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
                &download_cfg.version[1..download_cfg.version.len()].to_string(),
                architecture,
                vendor,
                os
            ))
        }
        _ => bail!("Unrecognized component: {}", download_cfg.name),
    }
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

pub fn download_file_and_unpack(download_cfg: &DownloadCfg) -> Result<()> {
    let tarball_name = tarball_name(download_cfg)?;
    let tarball_url = format!(
        "{}/{}/{}",
        &download_cfg.release_url, &download_cfg.version, &tarball_name
    );

    info!("Fetching binary from {}", &tarball_url);

    let fuelup_bin_dir = fuelup_bin_dir();
    let tarball_path = fuelup_bin_dir.join(tarball_name);

    if download_file(&tarball_url, &tarball_path).is_err() {
        error!("Failed to download from {}", &tarball_url,);
    };

    unpack(&tarball_path, &fuelup_bin_dir)?;

    Ok(())
}

pub fn unpack_extracted_bins(dir: &std::path::PathBuf) -> Result<()> {
    for entry in std::fs::read_dir(&dir)? {
        let sub_path = entry?.path();

        if sub_path.is_dir() {
            for bin in std::fs::read_dir(&sub_path)? {
                let bin_file = bin?;
                info!(
                    "Unpacking and moving {} to {}",
                    &bin_file.file_name().to_string_lossy(),
                    dir.display()
                );
                fs::copy(&bin_file.path(), dir.join(&bin_file.file_name()))?;
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

    #[test]
    fn test_unpack_extracted_bins() {
        let fuelup_bin_dir = tempfile::Builder::new()
            .prefix("mock-fuelup-bin")
            .tempdir()
            .unwrap();
        let mock_bin_dir = fuelup_bin_dir.path().join("forc-mock");
        let mock_bin_file_1 = mock_bin_dir.join("forc-mock-exec-1");
        let mock_bin_file_2 = mock_bin_dir.join("forc-mock-exec-2");

        fs::create_dir(&mock_bin_dir).unwrap();
        fs::File::create(mock_bin_file_1).unwrap();
        fs::File::create(mock_bin_file_2).unwrap();

        assert!(mock_bin_dir.exists());
        assert!(!fuelup_bin_dir.path().join("forc-mock-exec-1").exists());
        assert!(!fuelup_bin_dir.path().join("forc-mock-exec-2").exists());

        unpack_extracted_bins(&fuelup_bin_dir.path().to_path_buf()).unwrap();

        assert!(!mock_bin_dir.exists());
        assert!(fuelup_bin_dir.path().join("forc-mock-exec-1").exists());
        assert!(fuelup_bin_dir.path().join("forc-mock-exec-2").exists());
    }
}
