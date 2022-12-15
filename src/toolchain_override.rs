use anyhow::Result;
use serde::{Deserialize, Serialize};
use toml_edit::de;

#[derive(Debug, Deserialize, Serialize)]
pub struct ToolchainOverride {
    pub toolchain: ToolchainCfg,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ToolchainCfg {
    pub name: String,
    pub components: Option<Vec<String>>,
}

impl ToolchainOverride {
    pub(crate) fn parse(toml: &str) -> Result<Self> {
        let _override: ToolchainOverride = de::from_str(toml)?;
        Ok(_override)
    }
}
