use crate::{
    constants::{CHANNEL_LATEST_FILE_NAME, FUELUP_GH_PAGES},
    download::DownloadCfg,
};
use anyhow::{bail, Result};
use semver::Version;
use std::{collections::HashMap, str::FromStr};
use tempfile::tempdir_in;
use toml_edit::{Document, Item};
use tracing::error;

use crate::{download::download_file, file::read_file, path::fuelup_dir, toolchain::ToolchainName};

pub struct HashedBinary {
    pub url: String,
    pub hash: String,
}

impl HashedBinary {
    pub fn from_package(table: &Item) -> Result<Self> {
        // OK to unwrap since url and hash should be correctly created in the channel toml.
        let url = table["url"].as_str().unwrap();
        let hash = table["hash"].as_str().unwrap();
        Ok(Self {
            url: url.to_string(),
            hash: hash.to_string(),
        })
    }
}

pub struct Package {
    pub name: String,
    pub version: Version,
    pub targets: HashMap<String, HashedBinary>,
}

impl Package {
    pub fn from_channel(name: String, table: &Item) -> Result<Self> {
        let version = Version::from_str(
            table["version"]
                .as_str()
                .expect("Could not read 'version' from package"),
        )
        // OK to unwrap since version should be correctly created in the channel toml.
        .unwrap();
        let mut targets: HashMap<String, HashedBinary> = HashMap::new();
        for (target, target_table) in table["target"]
            .as_table()
            .expect("Could not read 'target' from package")
        {
            if let Ok(bin) = HashedBinary::from_package(target_table) {
                targets.insert(target.to_string(), bin);
            } else {
                error!(
                    "Could not create representation of binary for target {}",
                    target
                )
            }
        }

        Ok(Package {
            name,
            version,
            targets,
        })
    }
}

pub struct Channel {
    pub packages: Vec<Package>,
}

impl Channel {
    pub fn from_dist_channel(name: &ToolchainName) -> Result<Self> {
        let channel_url = match name {
            ToolchainName::Latest => FUELUP_GH_PAGES.to_owned() + CHANNEL_LATEST_FILE_NAME,
        };
        let fuelup_dir = fuelup_dir();
        let tmp_dir = tempdir_in(&fuelup_dir)?;
        let tmp_dir_path = tmp_dir.path();
        let toml = match download_file(&channel_url, &tmp_dir_path.join(CHANNEL_LATEST_FILE_NAME)) {
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
        let mut document = toml.parse::<Document>().expect("Invalid channel toml");

        let table = document.as_table_mut();
        let mut packages = Vec::new();

        for (name, package_table) in table["pkg"]
            .as_table()
            .expect("Failed to read pkg as table")
        {
            let package = Package::from_channel(name.to_string(), package_table)?;
            packages.push(package);
        }

        Ok(Self { packages })
    }

    pub fn build_download_configs(self) -> Vec<DownloadCfg> {
        self.packages
            .iter()
            .map(|p| {
                DownloadCfg::from_package(p)
                    .expect("Could not create DownloadCfg from a package parsed in latest channel")
            })
            .collect()
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

        assert_eq!(channel.packages.len(), 2);
        assert_eq!(channel.packages[0].name, "forc");
        assert_eq!(channel.packages[0].version, Version::new(0, 17, 0));

        assert_eq!(channel.packages[1].name, "fuel-core");
        assert_eq!(channel.packages[1].version, Version::new(0, 9, 4));
    }

    #[test]
    fn download_cfgs_from_channel() -> Result<()> {
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
        Ok(())
    }
}
