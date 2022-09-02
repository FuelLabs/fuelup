use crate::{
    constants::{
        CHANNEL_LATEST_FILE_NAME, CHANNEL_NIGHTLY_FILE_NAME, DATE_FORMAT_URL_FRIENDLY,
        FUELUP_GH_PAGES,
    },
    download::{download_file, DownloadCfg},
    file::read_file,
    toolchain::{DistToolchainName, OfficialToolchainDescription},
};
use anyhow::{bail, Result};
use semver::Version;
use serde::{Deserialize, Deserializer};
use sha2::{Digest, Sha256};
use std::{collections::HashMap, path::PathBuf};
use time::{macros::format_description, Date};
use toml_edit::de;

pub const LATEST: &str = "latest";
pub const STABLE: &str = "stable";
pub const BETA: &str = "beta";
pub const NIGHTLY: &str = "nightly";

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
    #[serde(deserialize_with = "version_and_date_from_str")]
    pub version: PackageVersion,
}

#[derive(Debug)]
pub struct PackageVersion {
    pub semver: Version,
    pub date: Option<Date>,
}

fn version_and_date_from_str<'de, D>(deserializer: D) -> Result<PackageVersion, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let (version, date) = match s.split_once(' ') {
        Some((version, date)) => (version, Some(date)),
        None => (s.as_str(), None),
    };

    let parsed_date = if let Some(date) = date {
        Date::parse(date, format_description!("([year]-[month]-[day])")).ok()
    } else {
        None
    };

    Ok(PackageVersion {
        semver: Version::parse(version).unwrap(),
        date: parsed_date,
    })
}

impl Channel {
    pub fn from_dist_channel(
        desc: &OfficialToolchainDescription,
        dst_path: PathBuf,
    ) -> Result<Self> {
        let channel_file_name = match desc.name {
            DistToolchainName::Latest => CHANNEL_LATEST_FILE_NAME,
            DistToolchainName::Nightly => CHANNEL_NIGHTLY_FILE_NAME,
        };

        let mut channel_url = FUELUP_GH_PAGES.to_owned();
        if desc.name == DistToolchainName::Nightly && desc.date.is_some() {
            channel_url.push_str("channels/nightly/");
            channel_url.push_str(&desc.date.unwrap().format(DATE_FORMAT_URL_FRIENDLY)?);
            channel_url.push('/');
        }

        channel_url.push_str(channel_file_name);
        let mut hasher = Sha256::new();
        let toml = match download_file(&channel_url, &dst_path.join(channel_file_name), &mut hasher)
        {
            Ok(_) => {
                let toml_path = dst_path.join(channel_file_name);
                read_file(channel_file_name, &toml_path)?
            }
            Err(_) => bail!(
                "Could not download {} to {}",
                &channel_url,
                dst_path.display()
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
    use crate::{constants::DATE_FORMAT, download::DownloadCfg, file::read_file};

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
            channel.pkg["forc"].version.semver,
            Version::parse("0.17.0").unwrap()
        );
        assert!(channel.pkg.contains_key("fuel-core"));
        assert_eq!(
            channel.pkg["fuel-core"].version.semver,
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
    fn download_cfgs_from_channel_latest() {
        let channel_path = std::env::current_dir()
            .unwrap()
            .join("tests/channel-fuel-latest-example.toml");
        let channel_file = read_file("channel-fuel-latest-example", &channel_path).unwrap();
        let channel = Channel::from_toml(&channel_file).unwrap();

        let cfgs: Vec<DownloadCfg> = channel.build_download_configs();

        assert_eq!(cfgs.len(), 2);
        assert_eq!(cfgs[0].name, "forc");
        assert_eq!(cfgs[0].version, Version::parse("0.17.0").unwrap());
        assert_eq!(cfgs[0].date, None);
        assert_eq!(cfgs[1].name, "fuel-core");
        assert_eq!(cfgs[1].version, Version::parse("0.9.4").unwrap());
        assert_eq!(cfgs[1].date, None);
    }

    #[test]
    fn download_cfgs_from_channel_nightly() {
        let channel_path = std::env::current_dir()
            .unwrap()
            .join("tests/channel-fuel-nightly-example.toml");
        let channel_file = read_file("channel-fuel-nightly-example", &channel_path).unwrap();
        let channel = Channel::from_toml(&channel_file).unwrap();

        let cfgs: Vec<DownloadCfg> = channel.build_download_configs();

        assert_eq!(cfgs.len(), 2);
        assert_eq!(cfgs[0].name, "forc");
        assert_eq!(cfgs[0].version, Version::parse("0.21.0-nightly").unwrap());
        assert_eq!(cfgs[0].date, Date::parse("2022-08-30", DATE_FORMAT).ok());
        assert_eq!(cfgs[1].name, "fuel-core");
        assert_eq!(cfgs[1].version, Version::parse("0.10.1-nightly").unwrap());
        assert_eq!(cfgs[1].date, Date::parse("2022-08-30", DATE_FORMAT).ok());
    }
}
