use anyhow::{bail, Result};
use semver::Version;
use serde::de::Error;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;
use std::str::FromStr;
use std::{collections::HashMap, path::PathBuf};
use time::Date;
use toml_edit::{de, ser, value, Document};
use tracing::{info, warn};

use crate::channel::{is_beta_toolchain, LATEST, NIGHTLY};
use crate::constants::{DATE_FORMAT, FUEL_TOOLCHAIN_TOML_FILE};
use crate::toolchain::{DistToolchainDescription, Toolchain};
use crate::{
    download::DownloadCfg, file, path::get_fuel_toolchain_toml, target_triple::TargetTriple,
};

// For composability with other functionality of fuelup, we want to add
// additional info to OverrideCfg (representation of 'fuel-toolchain.toml').
// In this case, we want the path to the toml file. More info might be
// needed in future.
#[derive(Debug, Deserialize, Serialize)]
pub struct ToolchainOverride {
    pub cfg: OverrideCfg,
    pub path: PathBuf,
}

// Representation of the entire 'fuel-toolchain.toml'.
#[derive(Debug, Deserialize, Serialize)]
pub struct OverrideCfg {
    pub toolchain: ToolchainCfg,
    pub components: Option<HashMap<String, Version>>,
}

// Represents the [toolchain] table in 'fuel-toolchain.toml'.
#[derive(Debug, Deserialize)]
pub struct ToolchainCfg {
    #[serde(deserialize_with = "deserialize_channel")]
    pub channel: Channel,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Channel {
    pub name: String,
    pub date: Option<Date>,
}

impl Serialize for ToolchainCfg {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut c = serializer.serialize_struct("ToolchainCfg", 2)?;
        c.serialize_field("channel", &self.channel.to_string())?;

        c.end()
    }
}

pub fn deserialize_channel<'de, D>(deserializer: D) -> Result<Channel, D::Error>
where
    D: Deserializer<'de>,
{
    let channel_str = String::deserialize(deserializer)?;

    channel_str.parse().map_or_else(
        |_| {
            Err(Error::invalid_value(
                serde::de::Unexpected::Str(&channel_str),
                &"one of <latest-YYYY-MM-DD|nightly-YYYY-MM-DD|beta-1|beta-2|beta-3>",
            ))
        },
        Result::Ok,
    )
}

impl fmt::Display for Channel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.date {
            Some(d) => write!(f, "{}-{}", self.name, d),
            None => write!(f, "{}", self.name),
        }
    }
}

impl FromStr for Channel {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        if is_beta_toolchain(s) {
            return Ok(Self {
                name: s.to_string(),
                date: None,
            });
        };

        if let Some((name, d)) = s.split_once('-') {
            Ok(Self {
                name: name.to_string(),
                date: Date::parse(d, DATE_FORMAT).ok(),
            })
        } else {
            if s == LATEST || s == NIGHTLY {
                bail!("'{s}' without date specifier is forbidden");
            }
            bail!("Invalid str for channel: '{}'", s);
        }
    }
}

impl ToolchainOverride {
    // Creates a representation of a 'fuel-toolchain.toml' from a file path.
    // This representation is an OverrideCfg and the file path.
    pub(crate) fn from_path(path: PathBuf) -> Result<Self> {
        let f = file::read_file(FUEL_TOOLCHAIN_TOML_FILE, path.as_path())?;
        let cfg: OverrideCfg = OverrideCfg::from_toml(&f)?;
        Ok(Self { cfg, path })
    }

    pub fn to_toml(&self) -> Document {
        let mut document = toml_edit::Document::new();

        document["toolchain"]["channel"] = value(self.cfg.toolchain.channel.to_string());
        if let Some(components) = &self.cfg.components {
            for (k, v) in components.iter() {
                document["components"][k] = value(v.to_string());
            }
        }
        document
    }

    pub fn from_project_root() -> Option<ToolchainOverride> {
        if let Some(fuel_toolchain_toml_file) = get_fuel_toolchain_toml() {
            match ToolchainOverride::from_path(fuel_toolchain_toml_file) {
                Ok(to) => Some(to),
                Err(e) => {
                    warn!("warning: invalid 'fuel-toolchain.toml' in project root: {e}");
                    None
                }
            }
        } else {
            None
        }
    }

    pub fn get_component_version(&self, component: &str) -> Option<&Version> {
        if let Some(components) = &self.cfg.components {
            components.get(component)
        } else {
            None
        }
    }

    pub fn install_missing_components(&self, toolchain: &Toolchain, called: &str) -> Result<()> {
        match &self.cfg.components {
            None => warn!(
                "warning: overriding toolchain '{}' in {} does not have any components listed",
                &self.cfg.toolchain.channel, FUEL_TOOLCHAIN_TOML_FILE
            ),
            Some(components) => {
                for component in components.keys() {
                    if !toolchain.has_component(component) {
                        let target_triple = TargetTriple::from_component(component)?;

                        if let Ok(download_cfg) = DownloadCfg::new(called, target_triple, None) {
                            info!(
                                "installing missing component '{}' specified in {}",
                                component, FUEL_TOOLCHAIN_TOML_FILE
                            );
                            toolchain.add_component(download_cfg)?;
                        };
                    }
                }
            }
        };
        Ok(())
    }
}

impl OverrideCfg {
    pub fn new(toolchain: ToolchainCfg, components: Option<HashMap<String, Version>>) -> Self {
        Self {
            toolchain,
            components,
        }
    }

    // Creates a representation of a 'fuel-toolchain.toml' from a toml string.
    // This is used in the implementation of ToolchainOverride, which is just
    // an OverrideCfg with its file path.
    pub(crate) fn from_toml(toml: &str) -> Result<Self> {
        let cfg: OverrideCfg = de::from_str(toml)?;
        if DistToolchainDescription::from_str(&cfg.toolchain.channel.to_string()).is_err() {
            bail!("Invalid channel '{}'", &cfg.toolchain.channel)
        }

        if let Some(components) = cfg.components.as_ref() {
            if components.is_empty() {
                bail!("'[components]' table is declared with no components")
            }
        }

        Ok(cfg)
    }

    pub fn to_string_pretty(self) -> Result<String, ser::Error> {
        ser::to_string_pretty(&self)
    }
}

#[cfg(test)]
mod tests {
    use crate::channel::{BETA_1, BETA_2, BETA_3, NIGHTLY};

    use super::*;

    #[test]
    fn parse_toolchain_override_latest_with_date() {
        const TOML: &str = r#"[toolchain]
channel = "latest-2023-01-09"
"#;
        let cfg = OverrideCfg::from_toml(TOML).unwrap();

        assert_eq!(cfg.toolchain.channel.to_string(), "latest-2023-01-09");

        assert!(cfg.components.is_none());
        assert_eq!(TOML, cfg.to_string_pretty().unwrap());
    }

    #[test]
    fn parse_toolchain_override_nightly_with_date() {
        const TOML: &str = r#"[toolchain]
channel = "nightly-2023-01-09"

[components]
forc = "0.33.0"
"#;
        let cfg = OverrideCfg::from_toml(TOML).unwrap();

        assert_eq!(cfg.toolchain.channel.to_string(), "nightly-2023-01-09");
        assert_eq!(
            cfg.components.as_ref().unwrap().get("forc").unwrap(),
            &Version::new(0, 33, 0)
        );
        assert_eq!(TOML, cfg.to_string_pretty().unwrap());
    }

    #[test]
    fn parse_toolchain_override_channel_without_date_error() {
        const LATEST: &str = r#"[toolchain]
channel = "latest"
"#;
        const NIGHTLY: &str = r#"[toolchain]
channel = "nightly"
"#;

        let result = OverrideCfg::from_toml(LATEST);
        assert!(result.is_err());
        let e = result.unwrap_err();
        assert_eq!(e
            .to_string(),
            "invalid value: string \"latest\", expected one of <latest-YYYY-MM-DD|nightly-YYYY-MM-DD|beta-1|beta-2|beta-3> for key `toolchain.channel`".to_string());

        let result = OverrideCfg::from_toml(NIGHTLY);
        assert!(result.is_err());
        let e = result.unwrap_err();

        assert_eq!(e
            .to_string(),
            "invalid value: string \"nightly\", expected one of <latest-YYYY-MM-DD|nightly-YYYY-MM-DD|beta-1|beta-2|beta-3> for key `toolchain.channel`".to_string());
    }

    #[test]
    fn parse_toolchain_override_invalid_tomls() {
        const EMPTY_STR: &str = "";
        const EMPTY_TOOLCHAIN: &str = r#"[toolchain]
"#;
        const INVALID_CHANNEL: &str = r#"[toolchain]
channel = "invalid-channel"
"#;
        const EMPTY_COMPONENTS: &str = r#"[toolchain]
channel = "beta-2"

[components]
"#;

        for toml in [
            EMPTY_STR,
            EMPTY_TOOLCHAIN,
            INVALID_CHANNEL,
            EMPTY_COMPONENTS,
        ] {
            assert!(OverrideCfg::from_toml(toml).is_err());
        }
    }

    #[test]
    fn channel_from_str() {
        assert!(Channel::from_str(BETA_1).is_ok());
        assert!(Channel::from_str(BETA_2).is_ok());
        assert!(Channel::from_str(BETA_3).is_ok());
        assert!(Channel::from_str(NIGHTLY).is_err());
        assert!(Channel::from_str(LATEST).is_err());
    }
}
