use std::{collections::HashMap, str::FromStr};

use anyhow::{bail, Result};
use semver::Version;
use tempfile::tempdir_in;
use toml_edit::{Document, Item};
use tracing::error;

use crate::{download::download_file, file::read_file, path::fuelup_dir, toolchain::ToolchainName};

pub const FUELUP_GH_PAGES: &str = "https://raw.githubusercontent.com/FuelLabs/fuelup/gh-pages/";

#[derive(Debug)]
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

#[derive(Debug)]
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
    pub fn from_dist_channel(name: ToolchainName) -> Result<Self> {
        let channel_url = match name {
            ToolchainName::Latest => FUELUP_GH_PAGES.to_owned() + "channel-fuel-latest.toml",
        };
        let fuelup_dir = fuelup_dir();
        let tmp_dir = tempdir_in(&fuelup_dir)?;
        let tmp_dir_path = tmp_dir.path();
        let toml = match download_file(&channel_url, &tmp_dir_path.join("channel-fuel-latest.toml"))
        {
            Ok(_) => {
                let toml_path = tmp_dir_path.join("channel-fuel-latest.toml");
                read_file("channel-fuel-latest.toml", &toml_path)?
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file::read_file;

    #[test]
    fn test_channel() {
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

        assert!(channel_path.is_file());
    }
}
