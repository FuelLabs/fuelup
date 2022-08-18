use crate::{
    constants::{CHANNEL_LATEST_FILE_NAME, FUELUP_GH_PAGES},
    download::{download_file, DownloadCfg},
    file::read_file,
    path::fuelup_dir,
    toolchain::DistToolchainName,
};
use anyhow::{bail, Result};
use semver::Version;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use tempfile::tempdir_in;
use toml_edit::de;

pub const LATEST: &str = "latest";
pub const STABLE: &str = "stable";
pub const NIGHTLY: &str = "beta";
pub const BETA: &str = "nightly";

#[derive(Debug, Deserialize)]
pub struct HashedBinary {
    pub url: String,
    pub hash: String,
}

#[derive(Debug, Deserialize)]
pub struct Channel {
    pub pkg: HashMap<String, Package>,
}

#[derive(Debug, Deserialize)]
pub struct Package {
    pub target: HashMap<String, HashedBinary>,
    pub version: Version,
}

impl Channel {
    pub fn from_dist_channel(name: &DistToolchainName) -> Result<Self> {
        let channel_url = match name {
            DistToolchainName::Latest => FUELUP_GH_PAGES.to_owned() + CHANNEL_LATEST_FILE_NAME,
        };
        let fuelup_dir = fuelup_dir();
        let tmp_dir = tempdir_in(&fuelup_dir)?;
        let tmp_dir_path = tmp_dir.path();
        let mut hasher = Sha256::new();
        let toml = match download_file(
            &channel_url,
            &tmp_dir_path.join(CHANNEL_LATEST_FILE_NAME),
            &mut hasher,
        ) {
            Ok(_) => {
                let toml_path = tmp_dir_path.join(CHANNEL_LATEST_FILE_NAME);
                read_file(CHANNEL_LATEST_FILE_NAME, &toml_path)?
            }
            Err(_) => bail!(
                "Could not download {} to {}",
                &channel_url,
                tmp_dir_path.display()
            ),
        };

        Self::from_toml(&toml)
    }

    pub fn from_toml(toml: &str) -> Result<Self> {
        let channel: Channel = de::from_str(toml)?;
        Ok(channel)
    }

    pub fn build_download_configs(self) -> Vec<DownloadCfg> {
        let mut cfgs = self
            .pkg
            .into_iter()
            .map(|(name, package)| {
                DownloadCfg::from_package(&name, package)
                    .expect("Could not create DownloadCfg from a package parsed in latest channel")
            })
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
        assert_eq!(channel.pkg["forc"].version, Version::new(0, 17, 0));
        assert!(channel.pkg.contains_key("fuel-core"));
        assert_eq!(channel.pkg["fuel-core"].version, Version::new(0, 9, 4));

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
        assert_eq!(cfgs[0].version, Version::new(0, 17, 0));
        assert_eq!(cfgs[1].name, "fuel-core");
        assert_eq!(cfgs[1].version, Version::new(0, 9, 4));
    }
}
