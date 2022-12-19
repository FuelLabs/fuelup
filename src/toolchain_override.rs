use std::str::FromStr;

use anyhow::Result;
use semver::Version;
use serde::ser::SerializeSeq;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use toml_edit::{de, ser, Document};
use tracing::warn;

use crate::{
    download::DownloadCfg, file, ops::fuelup_component::add::split_versioned_component,
    path::get_fuel_toolchain_toml, target_triple::TargetTriple, toolchain::Toolchain,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct ToolchainOverride {
    pub toolchain: ToolchainCfg,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ToolchainCfg {
    pub name: String,

    #[serde(default)]
    #[serde(deserialize_with = "deserialize_component")]
    #[serde(serialize_with = "serialize_component")]
    pub components: Option<Vec<Component>>,
}

#[derive(Debug, Serialize)]
pub struct Component {
    pub name: String,
    pub version: Option<Version>,
}

impl Component {
    pub fn new(name: String, version: Option<Version>) -> Self {
        return Self { name, version };
    }
}

fn deserialize_component<'de, D>(deserializer: D) -> Result<Option<Vec<Component>>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<Vec<String>> = Option::deserialize(deserializer)?;
    if let Some(s) = s {
        let mut res = vec![];
        for maybe_versioned_component in s {
            match Component::from_str(&maybe_versioned_component) {
                Ok(component) => res.push(component),
                Err(e) => return Err(serde::de::Error::custom(e)),
            }
        }

        Ok(Some(res))
    } else {
        Ok(None)
    }
}

fn serialize_component<S>(
    components: &Option<Vec<Component>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match *components {
        Some(ref components) => {
            let mut seq = serializer.serialize_seq(Some(components.len()))?;
            for component in components {
                let component_str = match &component.version {
                    Some(v) => format!("{}@{}", component.name, v),
                    None => component.name.clone(),
                };
                seq.serialize_element(&component_str)?;
            }

            seq.end()
        }
        None => serializer.serialize_none(),
    }
}

impl FromStr for Component {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (component, version) = split_versioned_component(s)?;

        Ok(Component {
            name: component,
            version,
        })
    }
}

impl ToolchainCfg {
    pub fn new(name: String, components: Option<Vec<Component>>) -> Self {
        return Self { name, components };
    }
}

impl ToolchainOverride {
    pub(crate) fn parse(toml: &str) -> Result<Self> {
        let toolchain_override: ToolchainOverride = de::from_str(toml)?;
        Ok(toolchain_override)
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
                    if !toolchain.has_component(&component.name) {
                        let target_triple = TargetTriple::from_component(&component.name).unwrap_or_else(|_| {
                            panic!("Failed to create target triple for '{}'", component.name)
                        });

                        if let Ok(download_cfg) = DownloadCfg::new(called, target_triple, component.version.clone()) {
                            toolchain.add_component(download_cfg).unwrap_or_else(|_| {
                                panic!(
                                    "Failed to add component '{}' to toolchain '{}'",
                                    component.name, toolchain.name,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_toml() -> Result<()> {
        let toml = r#"
[toolchain]
name = "latest"
"#;

        let toml_2 = r#"
[toolchain]
name = "nightly"
components = []
"#;

        let toml_3 = r#"
[toolchain]
name = "beta-2"
components = ["forc"]
"#;

        let toml_4 = r#"
[toolchain]
name = "my-toolchain"
components = ["forc@0.31.3"]
"#;

        ToolchainOverride::parse(toml)?;
        ToolchainOverride::parse(toml_2)?;
        ToolchainOverride::parse(toml_3)?;
        ToolchainOverride::parse(toml_4)?;
        Ok(())
    }

    #[test]
    fn parse_invalid_toml() -> Result<()> {
        let toml_empty = r#""#;
        let toml_no_name = r#"
[toolchain]
"#;
        let toml_invalid_semver = r#"
[toolchain]
name = "latest"
components = ["forc@0.31."]
        "#;

        for toml in [toml_no_name, toml_empty, toml_invalid_semver] {
            assert!(ToolchainOverride::parse(toml)
                .map_err(|e| e.to_string())
                .is_err());
        }

        Ok(())
    }
}
