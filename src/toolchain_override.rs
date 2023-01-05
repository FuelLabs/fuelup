use std::{collections::HashMap, path::PathBuf};

use anyhow::Result;
use semver::Version;
use serde::{Deserialize, Serialize};
use toml_edit::{de, ser, Document};
use tracing::{info, warn};

use crate::{
    constants::FUEL_TOOLCHAIN_TOML_FILE, download::DownloadCfg, file,
    path::get_fuel_toolchain_toml, target_triple::TargetTriple, toolchain::Toolchain,
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
}

impl OverrideCfg {
    pub(crate) fn from_toml(toml: &str) -> Result<Self> {
        let channel: OverrideCfg = de::from_str(toml)?;
        Ok(channel)
    }
}

impl ToolchainOverride {
    pub(crate) fn parse(toml: &str, path: PathBuf) -> Result<Self> {
        let cfg: OverrideCfg = OverrideCfg::from_toml(toml)?;

        Ok(ToolchainOverride { cfg, path })
    }

    pub(crate) fn to_toml(&self) -> std::result::Result<Document, ser::Error> {
        ser::to_document(&self)
    }

    pub fn to_string(&self) -> Result<String> {
        Ok(self.to_toml()?.to_string())
    }

    pub fn from_project_root() -> Option<ToolchainOverride> {
        if let Some(fuel_toolchain_toml_file) = get_fuel_toolchain_toml() {
            match file::read_file(FUEL_TOOLCHAIN_TOML_FILE, &fuel_toolchain_toml_file) {
                Ok(f) => ToolchainOverride::parse(&f, fuel_toolchain_toml_file.to_path_buf())
                    .map(Option::Some)
                    .unwrap_or_else(|_| {
                        warn!(
                            "Failed parsing {} at project root, using default toolchain instead",
                            FUEL_TOOLCHAIN_TOML_FILE
                        );
                        None
                    }),
                Err(_) => None,
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

        let toolchain_override = ToolchainOverride::parse(TOML, PathBuf::new()).unwrap();

        assert_eq!(toolchain_override.cfg.toolchain.channel, "latest");
        assert!(toolchain_override.cfg.components.is_none());
    }

    #[test]
    fn parse_toolchain_override_components() {
        const TOML: &str = r#"
[toolchain]
channel = "latest"

[components]
fuel-core = "0.15.1"
"#;

        let toolchain_override = ToolchainOverride::parse(TOML, PathBuf::new()).unwrap();

        assert_eq!(toolchain_override.cfg.toolchain.channel, "latest");
        assert_eq!(
            toolchain_override
                .cfg
                .components
                .as_ref()
                .unwrap()
                .keys()
                .len(),
            1
        );
        assert_eq!(
            toolchain_override
                .cfg
                .components
                .unwrap()
                .get("fuel-core")
                .unwrap(),
            &Version::new(0, 15, 1)
        );
    }

    #[test]
    fn parse_toolchain_override_invalid_tomls() {
        const EMPTY_STR: &str = "";
        const EMPTY_TOOLCHAIN: &str = r#"
[toolchain]
"#;

        for toml in [EMPTY_STR, EMPTY_TOOLCHAIN] {
            assert!(ToolchainOverride::parse(toml, PathBuf::new()).is_err());
        }
    }
}
