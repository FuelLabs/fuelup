use crate::{
    constants::{CHANNEL_LATEST_FILE_NAME, CHANNEL_NIGHTLY_FILE_NAME, FUELUP_GH_PAGES},
    download::{download_file, DownloadCfg},
    file::read_file,
    toolchain::{DistToolchainName, OfficialToolchainDescription},
};
use anyhow::{bail, Result};
use semver::Version;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fmt::Debug, path::PathBuf};
use toml_edit::{de, value, Document};

pub const LATEST: &str = "latest";
pub const STABLE: &str = "stable";
pub const BETA: &str = "beta";
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

fn implicit_table() -> toml_edit::Item {
    let mut tbl = toml_edit::Table::new();
    tbl.set_implicit(true);
    toml_edit::Item::Table(tbl)
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
        let channel_url = FUELUP_GH_PAGES.to_owned() + channel_file_name;
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

    pub fn into_toml(&self) -> Result<Document> {
        let mut document = toml_edit::Document::new();
        document["pkg"] = implicit_table();

        for (component, package) in &self.pkg {
            document["pkg"][&component] = implicit_table();
            document["pkg"][&component]["version"] = value(package.version.to_string());

            document["pkg"][&component]["target"] = implicit_table();

            for (target, bin) in &package.target {
                document["pkg"][&component]["target"][target.to_string()] = implicit_table();
                document["pkg"][&component]["target"][target.to_string()]["url"] =
                    value(bin.url.clone());
                document["pkg"][&component]["target"][target.to_string()]["hash"] =
                    value(bin.hash.clone());
            }
        }

        Ok(document)
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
    use std::fs;

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

    #[test]
    fn channel_into_toml() -> Result<()> {
        let channel_path = std::env::current_dir()
            .unwrap()
            .join("tests/channel-fuel-nightly-example.toml");
        let channel_file = read_file("channel-fuel-nightly-example", &channel_path).unwrap();
        let channel = Channel::from_toml(&channel_file).unwrap();

        let toml = channel.into_toml()?;
        let path = std::env::current_dir().unwrap().join("tests/chan.toml");

        fs::write(path, toml.to_string())?;

        assert_eq!(channel_file.trim(), toml.to_string().trim());

        Ok(())
    }
}
