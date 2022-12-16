use crate::{
    constants::{
        CHANNEL_APPS_REPO_FILE_NAME, CHANNEL_BETA_1_FILE_NAME, CHANNEL_BETA_2_FILE_NAME,
        CHANNEL_LATEST_FILE_NAME, CHANNEL_NIGHTLY_FILE_NAME, DATE_FORMAT_URL_FRIENDLY,
        FUELUP_GH_PAGES,
    },
    download::{download, DownloadCfg},
    toolchain::{DistToolchainDescription, DistToolchainName},
};
use anyhow::{bail, Result};
use component::Components;
use semver::Version;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fmt::Debug};
use toml_edit::de;
use tracing::warn;

pub const LATEST: &str = "latest";
pub const STABLE: &str = "stable";
pub const APPS_REPO: &str = "apps-repo";
pub const BETA_1: &str = "beta-1";
pub const BETA_2: &str = "beta-2";
pub const NIGHTLY: &str = "nightly";

#[derive(Debug, Deserialize, Serialize)]
pub struct HashedBinary {
    pub url: String,
    pub hash: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Channel {
    pub pkg: BTreeMap<String, Package>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Package {
    pub target: BTreeMap<String, HashedBinary>,
    pub version: Version,
}

pub fn is_beta_toolchain(name: &str) -> bool {
    name == BETA_1 || name == BETA_2
}

impl Channel {
    /// The returned `String` is a sha256 hash of the downloaded toolchain TOML bytes.
    pub fn from_dist_channel(desc: &DistToolchainDescription) -> Result<(Self, String)> {
        let channel_file_name = match desc.name {
            DistToolchainName::Latest => CHANNEL_LATEST_FILE_NAME,
            DistToolchainName::Nightly => CHANNEL_NIGHTLY_FILE_NAME,
            DistToolchainName::Beta1 => CHANNEL_BETA_1_FILE_NAME,
            DistToolchainName::Beta2 => CHANNEL_BETA_2_FILE_NAME,
            DistToolchainName::AppsRepo => CHANNEL_APPS_REPO_FILE_NAME,
        };

        let mut channel_url = FUELUP_GH_PAGES.to_owned();
        if desc.name == DistToolchainName::Nightly && desc.date.is_some() {
            channel_url.push_str("channels/nightly/");
            channel_url.push_str(&desc.date.unwrap().format(DATE_FORMAT_URL_FRIENDLY)?);
            channel_url.push('/');
        }

        channel_url.push_str(channel_file_name);
        let mut hasher = Sha256::new();
        let toml = match download(&channel_url, &mut hasher) {
            Ok(t) => String::from_utf8(t)?,
            Err(_) => bail!("Could not read {}", &channel_url),
        };

        let actual_hash = format!("{:x}", hasher.finalize());
        Ok((Self::from_toml(&toml)?, actual_hash))
    }

    pub fn from_toml(toml: &str) -> Result<Self> {
        let channel: Channel = de::from_str(toml)?;
        Ok(channel)
    }

    pub fn build_download_configs(self) -> Vec<DownloadCfg> {
        let mut cfgs = self
            .pkg
            .into_iter()
            .filter(|(component_name, _)| Components::contains_published(component_name))
            .map(|(name, package)| {
                DownloadCfg::from_package(&name, package).map_err(|_| {
                    warn!(
                        "Failed to recognize component: '{}'.
If this component should be downloadable, try running `fuelup self update` and re-run the installation.",
                        &name
                    )
                })
            })
            .filter_map(Result::ok)
            .collect::<Vec<DownloadCfg>>();
        cfgs.sort_by(|a, b| a.name.cmp(&b.name));
        cfgs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{download::DownloadCfg, file::read_file};

    #[test]
    fn channel_from_toml() {
        let channel_path = std::env::current_dir()
            .unwrap()
            .join("tests/channel-fuel-latest-example.toml");
        let channel_file = read_file("channel-fuel-latest-example", &channel_path).unwrap();
        let channel = Channel::from_toml(&channel_file).unwrap();

        assert_eq!(channel.pkg.keys().len(), 2);
        assert!(channel.pkg.contains_key("forc"));
        assert_eq!(
            channel.pkg["forc"].version,
            Version::parse("0.17.0").unwrap()
        );
        assert!(channel.pkg.contains_key("fuel-core"));
        assert_eq!(
            channel.pkg["fuel-core"].version,
            Version::parse("0.9.4").unwrap()
        );

        let targets = &channel.pkg["forc"].target;
        assert_eq!(targets.len(), 4);

        for target in targets.keys() {
            assert!(!targets[target].url.is_empty());
            assert!(!targets[target].hash.is_empty());
        }
        assert!(targets.contains_key("darwin_amd64"));
        assert!(targets.contains_key("darwin_arm64"));
        assert!(targets.contains_key("linux_amd64"));
        assert!(targets.contains_key("linux_arm64"));

        let targets = &channel.pkg["fuel-core"].target;
        assert_eq!(targets.len(), 4);

        for target in targets.keys() {
            assert!(!targets[target].url.is_empty());
            assert!(!targets[target].hash.is_empty());
        }
        assert!(targets.contains_key("aarch64-apple-darwin"));
        assert!(targets.contains_key("aarch64-unknown-linux-gnu"));
        assert!(targets.contains_key("x86_64-apple-darwin"));
        assert!(targets.contains_key("x86_64-unknown-linux-gnu"));
    }

    #[test]
    fn channel_from_toml_nightly() {
        let channel_path = std::env::current_dir()
            .unwrap()
            .join("tests/channel-fuel-nightly-example.toml");
        let channel_file = read_file("channel-fuel-nightly-example", &channel_path).unwrap();
        let channel = Channel::from_toml(&channel_file).unwrap();

        assert_eq!(channel.pkg.keys().len(), 2);
        assert!(channel.pkg.contains_key("forc"));
        assert_eq!(
            channel.pkg["forc"].version,
            Version::parse("0.24.3+nightly.20220915.0b69f4d4").unwrap()
        );
        assert!(channel.pkg.contains_key("fuel-core"));
        assert_eq!(
            channel.pkg["fuel-core"].version,
            Version::parse("0.10.1+nightly.20220915.bd5901f").unwrap()
        );

        let targets = &channel.pkg["forc"].target;
        assert_eq!(targets.len(), 4);

        for target in targets.keys() {
            assert!(!targets[target].url.is_empty());
            assert!(!targets[target].hash.is_empty());
        }
        assert!(targets.contains_key("darwin_amd64"));
        assert!(targets.contains_key("darwin_arm64"));
        assert!(targets.contains_key("linux_amd64"));
        assert!(targets.contains_key("linux_arm64"));

        let targets = &channel.pkg["fuel-core"].target;
        assert_eq!(targets.len(), 4);

        for target in targets.keys() {
            assert!(!targets[target].url.is_empty());
            assert!(!targets[target].hash.is_empty());
        }
        assert!(targets.contains_key("aarch64-apple-darwin"));
        assert!(targets.contains_key("aarch64-unknown-linux-gnu"));
        assert!(targets.contains_key("x86_64-apple-darwin"));
        assert!(targets.contains_key("x86_64-unknown-linux-gnu"));
    }

    #[test]
    fn download_cfgs_from_channel() {
        let channel_path = std::env::current_dir()
            .unwrap()
            .join("tests/channel-fuel-latest-example.toml");
        let channel_file = read_file("channel-fuel-latest-example", &channel_path).unwrap();
        let channel = Channel::from_toml(&channel_file).unwrap();

        let cfgs: Vec<DownloadCfg> = channel.build_download_configs();

        assert_eq!(cfgs.len(), 2);
        assert_eq!(cfgs[0].name, "forc");
        assert_eq!(cfgs[0].version, Version::parse("0.17.0").unwrap());
        assert_eq!(cfgs[1].name, "fuel-core");
        assert_eq!(cfgs[1].version, Version::parse("0.9.4").unwrap());
    }
}
