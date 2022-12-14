use anyhow::Result;
use serde::{Deserialize, Serialize};
use toml_edit::{de, ser, Document};

#[derive(Debug, Deserialize, Serialize)]
pub struct ToolchainOverride {
    pub toolchain: ToolchainCfg,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ToolchainCfg {
    pub name: String,
}

impl ToolchainOverride {
    pub(crate) fn parse(toml: &str) -> Result<Self> {
        let _override: ToolchainOverride = de::from_str(toml)?;
        Ok(_override)
    }

    pub(crate) fn to_string(&self) -> Result<String> {
        Ok(self.to_toml()?.to_string())
    }

    pub(crate) fn to_toml(&self) -> std::result::Result<Document, ser::Error> {
        ser::to_document(&self)
    }
}
