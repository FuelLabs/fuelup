use anyhow::{anyhow, bail, Result};
use component::{Component, FUELUP};
use flate2::read::GzDecoder;
use indicatif::{FormattedDuration, HumanBytes, HumanDuration, ProgressBar, ProgressStyle};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Duration;
use std::{fs, thread};
use tar::Archive;
use tracing::{debug, error, info, warn};
use ureq::Response;

use crate::channel::Channel;
use crate::channel::Package;
use crate::constants::CHANNEL_LATEST_URL;
use crate::target_triple::TargetTriple;
use crate::toolchain::DistToolchainDescription;

fn github_releases_download_url(repo: &str, tag: &Version, tarball: &str) -> String {
    format!("https://github.com/FuelLabs/{repo}/releases/download/v{tag}/{tarball}")
}

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
            None => get_latest_version(name)
                .map_err(|e| anyhow!("Error getting latest tag for '{}': {}", name, e))?,
        };

        let (tarball_name, tarball_url) = if name == FUELUP {
            let tarball_name = tarball_name(FUELUP, &version, &target);
            let tarball_url = github_releases_download_url(FUELUP, &version, &tarball_name);
            (tarball_name, tarball_url)
        } else if let Ok(component) = Component::from_name(name) {
            let tarball_name = tarball_name(&component.tarball_prefix, &version, &target);
            let tarball_url =
                github_releases_download_url(&component.repository_name, &version, &tarball_name);
            (tarball_name, tarball_url)
        } else {
            bail!("Unrecognized component: {}", name)
        };

        Ok(Self {
            name: name.to_string(),
            target,
            version,
            tarball_name,
            tarball_url,
            hash: None,
        })
    }

    pub fn from_package(name: &str, package: &Package) -> Result<Self> {
        let target = TargetTriple::from_component(name)?;
        let tarball_name = tarball_name(name, &package.version, &target);
        let tarball_url = package.target[&target.to_string()].url.clone();
        let hash = Some(package.target[&target.to_string()].hash.clone());
        Ok(Self {
            name: name.to_string(),
            target,
            version: package.version.clone(),
            tarball_name,
            tarball_url,
            hash,
        })
    }
}

pub fn build_agent() -> Result<ureq::Agent> {
    let agent_builder = ureq::builder().user_agent("fuelup");
    let proxy_result = env::var("http_proxy")
        .or(env::var("HTTP_PROXY"))
        .or(env::var("https_proxy"))
        .or(env::var("HTTPS_PROXY"))
        .or(env::var("all_proxy"))
        .or(env::var("ALL_PROXY"));

    if let Ok(proxy) = proxy_result {
        return match ureq::Proxy::new(&proxy) {
            Ok(proxy) => Ok(agent_builder.proxy(proxy).build()),
            Err(err) => {
                error!("Failed to build proxy with http_proxy={}, {}", proxy, err);
                Err(err.into())
            }
        };
    }

    Ok(agent_builder.build())
}

pub fn tarball_name(tarball_prefix: &str, version: &Version, target: &TargetTriple) -> String {
    if tarball_prefix == "forc-binaries" {
        format!("{tarball_prefix}-{target}.tar.gz")
    } else {
        format!("{tarball_prefix}-{version}-{target}.tar.gz")
    }
}

pub fn get_latest_version(name: &str) -> Result<Version> {
    let handle = build_agent()?;

    let mut data = Vec::new();
    if name == FUELUP {
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

        if let Ok(channel) =
            Channel::from_dist_channel(&DistToolchainDescription::from_str("latest")?)
        {
            channel
                .pkg
                .get(name)
                .ok_or_else(|| {
                    anyhow!(
                        "'{name}' is not a valid, downloadable package in the 'latest' channel."
                    )
                })
                .map(|p| p.version.clone())
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

    fs::remove_file(tar_path)?;
    Ok(())
}

pub fn download(url: &str) -> Result<Vec<u8>> {
    const RETRY_ATTEMPTS: u8 = 4;
    const RETRY_DELAY_SECS: u64 = 3;

    let handle = build_agent()?;

    for _ in 1..RETRY_ATTEMPTS {
        match handle.get(url).call() {
            Ok(response) => {
                let mut data = Vec::new();
                write_response_with_progress_bar(response, &mut data, String::new())?;
                return Ok(data);
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

    bail!("Could not read file");
}

pub fn download_file(url: &str, path: &PathBuf) -> Result<()> {
    const RETRY_ATTEMPTS: u8 = 4;
    const RETRY_DELAY_SECS: u64 = 3;

    let handle = build_agent()?;

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;

    for _ in 1..RETRY_ATTEMPTS {
        match handle.get(url).call() {
            Ok(response) => {
                if let Err(e) = write_response_with_progress_bar(
                    response,
                    &mut file,
                    path.display().to_string(),
                ) {
                    fs::remove_file(path)?;
                    return Err(e);
                }
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
                fs::remove_file(path)?;
                // handle other status code and non-status code errors
                bail!("Unexpected error: {}", e.to_string());
            }
        }
    }

    fs::remove_file(path)?;
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

    if let Err(e) = download_file(&download_cfg.tarball_url, &tarball_path) {
        bail!(
            "Failed to download {} - {}. The release may not be ready yet.",
            &download_cfg.tarball_name,
            e
        );
    };

    unpack(&tarball_path, dst_dir_path)?;

    Ok(())
}

pub fn unpack_bins(dir: &Path, dst_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut downloaded: Vec<PathBuf> = Vec::new();
    for entry in std::fs::read_dir(dir)? {
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

                let dst_bin_file = dir.join(&bin_file_name);
                if dst_bin_file.exists() {
                    fs::remove_file(&dst_bin_file)?;
                }
                fs::copy(bin_file.path(), dst_bin_file)?;
                downloaded.push(dst_dir.join(bin_file_name));
            }

            fs::remove_dir_all(sub_path)?;
        }
    }

    Ok(downloaded)
}

/// write Ok(Response) to provided writer with progress bar displaying writing status
fn write_response_with_progress_bar<W: Write>(
    response: Response,
    writer: &mut W,
    target: String,
) -> Result<()> {
    let total_size = response
        .header("Content-Length")
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);
    let mut downloaded_size = 0;
    let mut buffer = [0; 8192];
    let progress_bar = ProgressBar::new(total_size);
    progress_bar.set_style(
                ProgressStyle::default_bar()
                    .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta}) - {msg:.green}")
                    .unwrap()
                    .progress_chars("##-"),
            );
    let mut reader = progress_bar.wrap_read(response.into_reader());

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        if let Err(e) = writer.write_all(&buffer[..bytes_read]) {
            log_progress_bar(&progress_bar);
            if target.is_empty() {
                bail!("Something went wrong writing data: {}", e)
            }
            bail!("Something went wrong writing data to {}: {}", target, e)
        };
        downloaded_size += bytes_read as u64;
        progress_bar.set_position(downloaded_size);
    }
    if total_size == 0 {
        // to be compatible with case total_size is 0
        // Note: there maybe a bug for ureq that in some case, it return response with empty value of "Content-Length". See `test_agent`
        progress_bar.set_length(downloaded_size);
    }
    progress_bar.finish_with_message("Download complete");
    log_progress_bar(&progress_bar);
    Ok(())
}

fn log_progress_bar(progress_bar: &ProgressBar) {
    debug!(
        "[{}] [{}] {}/{} ({}) - {}",
        FormattedDuration(progress_bar.elapsed()),
        "#".repeat(
            (progress_bar.position() * 40
                / progress_bar.length().unwrap_or(progress_bar.position())) as usize
        ),
        HumanBytes(progress_bar.position()),
        HumanBytes(progress_bar.length().unwrap_or(progress_bar.position())),
        HumanDuration(progress_bar.eta()),
        progress_bar.message(),
    );
}

/// Read the version (as a plain String) used by the `fuels` dependency, if it exists.
fn fuels_version_from_toml(toml: toml_edit::Document) -> Result<String> {
    if let Some(deps) = toml.get("dependencies") {
        if let Some(fuels) = deps.get("fuels") {
            let version = match fuels.as_value() {
                Some(toml_edit::Value::String(s)) => s.value().to_string(),
                Some(toml_edit::Value::InlineTable(t)) => t.get("version").map_or_else(
                    || "".to_string(),
                    |v| v.as_str().unwrap_or_default().to_string(),
                ),
                _ => String::default(),
            };

            return Ok(version);
        } else {
            bail!("'fuels' dependency does not exist");
        };
    };

    bail!("the table 'dependencies' does not exist");
}

/// Fetches the Cargo.toml of a component in its repository and tries to read the version of
/// the `fuels` dependency.
pub fn fetch_fuels_version(cfg: &DownloadCfg) -> Result<String> {
    let url = match cfg.name.as_str() {
        "forc" => format!(
            "https://raw.githubusercontent.com/FuelLabs/sway/v{}/test/src/sdk-harness/Cargo.toml",
            cfg.version
        ),
        "forc-wallet" => {
            format!(
                "https://raw.githubusercontent.com/FuelLabs/forc-wallet/v{}/Cargo.toml",
                cfg.version
            )
        }
        _ => bail!("invalid component to fetch fuels version for"),
    };

    let handle = if let Ok(proxy) = env::var("http_proxy") {
        ureq::builder()
            .user_agent("fuelup")
            .proxy(ureq::Proxy::new(proxy)?)
            .build()
    } else {
        ureq::builder().user_agent("fuelup").build()
    };

    if let Ok(resp) = handle.get(&url).call() {
        let cargo_toml = toml_edit::Document::from_str(&resp.into_string()?)?;
        return fuels_version_from_toml(cargo_toml);
    }

    bail!("Failed to get fuels version");
}

#[cfg(test)]
mod tests {
    use super::*;
    use dirs::home_dir;
    use std::io::{self, Result};
    use tempfile;

    struct MockWriter;

    impl Write for MockWriter {
        fn write_all(&mut self, _: &[u8]) -> Result<()> {
            Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "Mock Interrupted Error",
            ))
        }

        fn write(&mut self, _: &[u8]) -> Result<usize> {
            Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "Mock Interrupted Error",
            ))
        }

        fn flush(&mut self) -> Result<()> {
            Ok(())
        }
    }

    pub(crate) fn with_toolchain_dir<F>(f: F) -> Result<()>
    where
        F: FnOnce(tempfile::TempDir) -> Result<()>,
    {
        let toolchain_bin_dir = tempfile::tempdir()?;
        f(toolchain_bin_dir)
    }

    #[test]
    fn test_fuels_version_from_toml() {
        let toml = r#"
[package]        
name = "forc"

[dependencies]
fuels = "0.1"
"#;
        assert_eq!(
            "0.1",
            fuels_version_from_toml(toml_edit::Document::from_str(toml).unwrap()).unwrap()
        );

        let toml = r#"
[package]        
name = "forc"

[dependencies]
fuels = { version = "0.1", features = ["some-feature"] }
"#;

        assert_eq!(
            "0.1",
            fuels_version_from_toml(toml_edit::Document::from_str(toml).unwrap()).unwrap()
        );
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

    #[test]
    fn test_write_response_with_progress_bar() -> anyhow::Result<()> {
        let mut data = Vec::new();
        let len = 100;
        let body = "A".repeat(len);
        let s = format!(
            "HTTP/1.1 200 OK\r\n\
                 \r\n
                 {}",
            body,
        );
        let res = s.parse::<Response>().unwrap();
        assert!(write_response_with_progress_bar(res, &mut data, String::new()).is_ok());
        let written_res = String::from_utf8(data)?;
        assert!(written_res.trim().eq(&body));
        Ok(())
    }

    #[test]
    fn test_write_response_with_progress_bar_fail() {
        let mut mock_writer = MockWriter;
        let len = 9000;
        let body = "A".repeat(len);
        let s = format!(
            "HTTP/1.1 200 OK\r\n\
                 Content-Length: {}\r\n
                 \r\n
                 {}",
            len, body,
        );
        let res = s.parse::<Response>().unwrap();
        assert_eq!(
            write_response_with_progress_bar(res, &mut mock_writer, String::new())
                .unwrap_err()
                .to_string(),
            "Something went wrong writing data: Mock Interrupted Error"
        );
    }

    #[test]
    fn test_agent() -> anyhow::Result<()> {
        // this test case is used to illustrate the bug of ureq that sometimes doesn't return "Content-Length" header
        let handle = build_agent()?;
        let response = handle.get("https://raw.githubusercontent.com/FuelLabs/fuelup/gh-pages/channel-fuel-beta-4.toml").call()?;
        assert!(response.header("Content-Length").is_none());
        Ok(())
    }
}
