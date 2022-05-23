use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use curl::easy::Easy;
use dirs::home_dir;
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::time::Duration;
use tar::Archive;

use crate::constants::FUELUP_PATH;

#[derive(Debug, Serialize, Deserialize)]
struct LatestReleaseAPIResponse {
    url: String,
    tag_name: String,
    name: String,
}

pub fn forc_bin_tarball_name() -> Result<String> {
    let os = match std::env::consts::OS {
        "macos" => Ok("darwin"),
        "linux" => Ok("linux"),
        unsupported_os => Err(anyhow!("Unsupported os: {}", unsupported_os)),
    };
    let architecture = match std::env::consts::ARCH {
        "aarch64" => Ok("arm64"),
        "x86_64" => Ok("amd64"),
        unsupported_arch => Err(anyhow!("Unsupported architecture: {}", unsupported_arch)),
    };

    Ok(format!("forc-binaries-{}_{}.tar.gz", os?, architecture?))
}

pub fn fuel_core_bin_tarball_name(version: &str) -> Result<String> {
    let architecture = match std::env::consts::ARCH {
        "aarch64" => Ok("aarch64"),
        "x86_64" => Ok("x86_64"),
        unsupported_arch => Err(anyhow!("Unsupported architecture: {}", unsupported_arch)),
    };

    let vendor = match std::env::consts::OS {
        "macos" => "apple",
        _ => "unknown",
    };

    let os = match std::env::consts::OS {
        "macos" => Ok("darwin"),
        "linux" => Ok("linux-gnu"),
        unsupported_os => Err(anyhow!("Unsupported os: {}", unsupported_os)),
    };

    Ok(format!(
        "fuel-core-{}-{}-{}-{}.tar.gz",
        &version[1..version.len()],
        architecture?,
        vendor,
        os?
    ))
}

pub fn get_latest_tag(github_api_url: &str) -> Result<String> {
    let mut handle = Easy::new();

    handle.url(github_api_url)?;
    handle.connect_timeout(Duration::new(30, 0))?;
    handle.follow_location(true)?;
    handle.useragent("user-agent")?;

    let mut data = Vec::new();
    {
        let mut transfer = handle.transfer();
        transfer
            .write_function(|new_data| {
                data.extend_from_slice(new_data);
                Ok(new_data.len())
            })
            .unwrap();
        transfer.perform().unwrap();
    }

    let response: LatestReleaseAPIResponse = serde_json::from_str(&String::from_utf8_lossy(&data))?;
    Ok(response.tag_name)
}

fn fuelup_path() -> PathBuf {
    home_dir().unwrap().join(FUELUP_PATH)
}

fn unpack(tar_path: &Path, dst: &Path) -> Result<()> {
    let tar_gz = File::open(tar_path)?;
    let decompressed = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(decompressed);

    if let Err(e) = archive.unpack(dst) {
        eprintln!("{}", e);
    };

    Ok(())
}

pub fn download_file(url: &str, path: &PathBuf) -> Result<File> {
    let mut handle = Easy::new();

    let mut file = OpenOptions::new().write(true).create(true).open(&path)?;

    handle.url(url)?;
    handle.connect_timeout(Duration::new(30, 0))?;
    handle.follow_location(true)?;
    handle.useragent("user-agent")?;

    {
        let mut transfer = handle.transfer();
        transfer
            .write_function(|new_data| {
                file.write_all(new_data);
                Ok(new_data.len())
            })
            .unwrap();
        transfer.perform().unwrap();
    }

    Ok(file)
}

pub fn download_file_and_unpack(
    github_release_url: &str,
    tag: &str,
    tarball_name: &str,
) -> Result<()> {
    let tarball_url = format!("{}/{}/{}", &github_release_url, &tag, &tarball_name);

    println!("Fetching binary from {}", &tarball_url);

    let tarball_path = fuelup_path().join(tarball_name);

    download_file(&tarball_url, &tarball_path)?;
    let dst_path = home_dir().unwrap().join(Path::new(".fuelup"));

    unpack(&tarball_path, &dst_path)?;

    fs::remove_file(&tarball_path)?;

    Ok(())
}
