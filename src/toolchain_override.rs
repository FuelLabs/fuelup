use anyhow::Result;
use serde::{Deserialize, Serialize};
use toml_edit::{de, ser, Document};
use tracing::warn;

use crate::{
    download::DownloadCfg, file, path::get_fuel_toolchain_toml, target_triple::TargetTriple,
    toolchain::Toolchain,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct ToolchainOverride {
    pub toolchain: ToolchainCfg,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ToolchainCfg {
    pub name: String,
    pub components: Option<Vec<String>>,
}

impl ToolchainCfg {
    pub fn new(name: String, components: Option<Vec<String>>) -> Self {
        Self { name, components }
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
            match file::read_file("fuel-toolchain", &fuel_toolchain_toml_file) {
                Ok(f) => ToolchainOverride::parse(&f)
                    .map(Option::Some)
                    .expect("Failed parsing fuel-toolchain.toml at project root"),
                Err(_) => None,
            }
        } else {
            None
        }
    }

    pub fn install_components(&self, toolchain: &Toolchain, called: &str) -> Result<()> {
        match self.toolchain.components.as_deref() {
            Some([]) | None => warn!(
                "warning: overriding toolchain '{}' in fuel-toolchain.toml does not have any components listed",
                &self.toolchain.name
            ),
            Some(components) => {
                for component in components {
                    if !toolchain.has_component(component) {
                        let target_triple = TargetTriple::from_component(component).unwrap_or_else(|_| {
                            panic!("Failed to create target triple for '{}'", component)
                        });

                        if let Ok(download_cfg) = DownloadCfg::new(called, target_triple, None) {
                            toolchain.add_component(download_cfg).unwrap_or_else(|_| {
                                panic!(
                                    "Failed to add component '{}' to toolchain '{}'",
                                    component, toolchain.name,
                                )
                            });
                        }
                    }
                }
            }
        };
        Ok(())
    }
}
