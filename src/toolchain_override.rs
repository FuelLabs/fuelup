use anyhow::Result;
use serde::{Deserialize, Serialize};
use toml_edit::{de, ser, Document};
use tracing::{info, warn};

use crate::{
    constants::FUEL_TOOLCHAIN_TOML_FILE, download::DownloadCfg, file,
    path::get_fuel_toolchain_toml, target_triple::TargetTriple, toolchain::Toolchain,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct ToolchainOverride {
    pub toolchain: ToolchainCfg,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ToolchainCfg {
    pub channel: String,
    pub components: Option<Vec<String>>,
}

impl ToolchainCfg {
    pub fn new(channel: String, components: Option<Vec<String>>) -> Self {
        Self {
            channel,
            components,
        }
    }
}

impl ToolchainOverride {
    pub(crate) fn parse(toml: &str) -> Result<Self> {
        let _override: ToolchainOverride = de::from_str(toml)?;
        Ok(_override)
    }

    pub(crate) fn to_toml(&self) -> std::result::Result<Document, ser::Error> {
        ser::to_document(&self)
    }

    pub fn to_string(&self) -> Result<String> {
        Ok(self.to_toml()?.to_string())
    }

    pub fn from_file() -> Option<ToolchainOverride> {
        if let Some(fuel_toolchain_toml_file) = get_fuel_toolchain_toml() {
            match file::read_file(FUEL_TOOLCHAIN_TOML_FILE, &fuel_toolchain_toml_file) {
                Ok(f) => ToolchainOverride::parse(&f)
                    .map(Option::Some)
                    .expect(&format!(
                        "Failed parsing {} at project root",
                        FUEL_TOOLCHAIN_TOML_FILE
                    )),
                Err(_) => None,
            }
        } else {
            None
        }
    }

    pub fn install_missing_components(&self, toolchain: &Toolchain, called: &str) -> Result<()> {
        match self.toolchain.components.as_deref() {
            Some([]) | None => warn!(
                "warning: overriding toolchain '{}' in {} does not have any components listed",
                &self.toolchain.channel, FUEL_TOOLCHAIN_TOML_FILE
            ),
            Some(components) => {
                for component in components {
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
