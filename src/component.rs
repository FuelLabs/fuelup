use std::collections::HashMap;

use anyhow::Result;
use serde::Deserialize;
use toml_edit::de;

pub const FORC: &str = "forc";
pub const FUEL_CORE: &str = "fuel-core";
pub const FUELUP: &str = "fuelup";
pub const FORC_EXPLORE: &str = "forc-explore";
pub const FORC_FMT: &str = "forc-fmt";
pub const FORC_LSP: &str = "forc-lsp";
pub const FORC_DEPLOY: &str = "forc-run";
pub const FORC_RUN: &str = "forc-deploy";

pub const SUPPORTED_PLUGINS: &[&str] = &[FORC_FMT, FORC_LSP, FORC_EXPLORE, FORC_DEPLOY, FORC_RUN];

const COMPONENTS_TOML: &'static str = include_str!("../components.toml");

#[derive(Debug, Deserialize)]
pub struct Components {
    pub component: HashMap<String, Component>,
}

#[derive(Debug, Deserialize)]
pub struct Component {
    pub name: String,
    pub is_plugin: Option<bool>,
    pub tarball_prefix: String,
    pub executables: Vec<String>,
    pub repository_url: String,
    pub targets: Vec<String>,
}

#[derive(Debug)]
pub struct Plugin {
    pub name: String,
    pub executables: Vec<String>,
}

impl Plugin {
    pub fn is_main_executable(&self) -> bool {
        self.executables.len() == 1 && self.name == self.executables[0]
    }
}

impl Components {
    pub fn from_toml(toml: &str) -> Result<Self> {
        let components: Components = de::from_str(toml)?;
        Ok(components)
    }

    pub fn collect_plugins() -> Result<Vec<Plugin>> {
        let components = Self::from_toml(COMPONENTS_TOML)?;

        let plugins = components
            .component
            .keys()
            .filter(|&c| {
                components
                    .component
                    .get(c)
                    .map_or(false, |p| p.is_plugin.unwrap_or_default())
            })
            .map(|p| {
                let plugin = components.component.get(p).expect("Failed to get p");
                Plugin {
                    name: plugin.name.clone(),
                    executables: plugin.executables.clone(),
                }
            })
            .collect();

        Ok(plugins)
    }

    pub fn collect_plugin_executables() -> Result<Vec<String>> {
        let plugins = Self::collect_plugins()?;
        let mut executables = vec![];

        for plugin in plugins.iter() {
            executables.extend(plugin.executables.clone().into_iter())
        }

        Ok(executables)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOML: &str = r#"
[component.forc-fmt]
name = "forc-fmt"
is_plugin = true
tarball_prefix = "forc-binaries"
executables = ["forc-fmt"]
repository_url = "https://github.com/FuelLabs/sway"
targets = ["linux_amd64", "linux_arm64", "darwin_amd64", "darwin_arm64"]
"#;

    #[test]
    fn test_toml() -> Result<()> {
        let components = Components::from_toml(TOML)?;

        assert_eq!(components.component["forc-fmt"].name, "forc-fmt");
        assert_eq!(components.component["forc-fmt"].is_plugin, Some(true));
        assert_eq!(
            components.component["forc-fmt"].tarball_prefix,
            "forc-binaries"
        );
        assert_eq!(components.component["forc-fmt"].executables, ["forc-fmt"]);
        assert_eq!(
            components.component["forc-fmt"].repository_url,
            "https://github.com/FuelLabs/sway"
        );
        assert_eq!(
            components.component["forc-fmt"].targets,
            ["linux_amd64", "linux_arm64", "darwin_amd64", "darwin_arm64"]
        );

        Ok(())
    }

    #[test]
    fn test_collect_plugins() {
        assert!(Components::collect_plugins().is_ok());
    }

    #[test]
    fn test_collect_plugin_executables() {
        assert!(Components::collect_plugin_executables().is_ok());
    }
}
