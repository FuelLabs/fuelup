use anyhow::{bail, Result};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::{collections::HashMap, path::PathBuf};
use toml_edit::{de, ser, value, Document};
use tracing::{info, warn};

use crate::{
    constants::FUEL_TOOLCHAIN_TOML_FILE,
    download::DownloadCfg,
    file,
    path::get_fuel_toolchain_toml,
    target_triple::TargetTriple,
    toolchain::{DistToolchainName, Toolchain},
};

#[derive(Debug, Deserialize, Serialize)]
pub struct ToolchainOverride {
    pub cfg: OverrideCfg,
    pub path: PathBuf,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OverrideCfg {
    pub toolchain: ToolchainCfg,
    pub components: Option<HashMap<String, Version>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ToolchainCfg {
    pub channel: String,
}

impl OverrideCfg {
    pub fn new(toolchain: ToolchainCfg, components: Option<HashMap<String, Version>>) -> Self {
        Self {
            toolchain,
            components,
        }
    }

    pub fn to_string(&self) -> String {
        ser::to_string(&self).unwrap()
    }

    pub fn to_document(self) -> Document {
        let mut document = toml_edit::Document::new();

        document["toolchain"] = toml_edit::Item::Table(toml_edit::Table::new());
        document["toolchain"]["channel"] = value(self.toolchain.channel);
        if let Some(components) = self.components {
            document["components"] = toml_edit::Item::Table(toml_edit::Table::new());
            for (k, v) in components.iter() {
                document["components"][k] = value(v.to_string());
            }
        }

        return document;
    }
}

impl OverrideCfg {
    pub(crate) fn from_toml(toml: &str) -> Result<Self> {
        let cfg: OverrideCfg = de::from_str(toml)?;
        if DistToolchainName::from_str(&cfg.toolchain.channel).is_ok() {
            Ok(cfg)
        } else {
            bail!("Invalid channel '{}'", &cfg.toolchain.channel)
        }
    }
}

impl ToolchainOverride {
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
                    warn!("warning: Invalid 'fuel-toolchain.toml' exists in project root: {e}");
                    None
                }
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_toolchain_override_channel_only() {
        const TOML: &str = r#"
[toolchain]
channel = "latest"
"#;

        let cfg = OverrideCfg::from_toml(TOML).unwrap();

        assert_eq!(cfg.toolchain.channel, "latest");
        assert!(cfg.components.is_none());
        assert_eq!(TOML, cfg.to_document().to_string());
    }

    #[test]
    fn parse_toolchain_override_components() {
        const TOML: &str = r#"
[toolchain]
channel = "latest"

[components]
fuel-core = "0.15.1"
"#;

        let cfg = OverrideCfg::from_toml(TOML).unwrap();

        assert_eq!(cfg.toolchain.channel, "latest");
        assert_eq!(cfg.components.as_ref().unwrap().keys().len(), 1);
        assert_eq!(
            cfg.components.as_ref().unwrap().get("fuel-core").unwrap(),
            &Version::new(0, 15, 1)
        );
        assert_eq!(TOML, cfg.to_document().to_string());
    }

    #[test]
    fn parse_toolchain_override_invalid_tomls() {
        const EMPTY_STR: &str = "";
        const EMPTY_TOOLCHAIN: &str = r#"
[toolchain]
"#;
        const INVALID_CHANNEL: &str = r#"
[toolchain]
channel = "invalid-channel"
"#;

        for toml in [EMPTY_STR, EMPTY_TOOLCHAIN, INVALID_CHANNEL] {
            assert!(OverrideCfg::from_toml(toml).is_err());
        }
    }
}
