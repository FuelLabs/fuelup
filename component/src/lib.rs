use std::collections::HashMap;

use anyhow::{anyhow, Result};
use serde::Deserialize;
use toml_edit::de;

// Keeping forc since some ways we handle forc is slightly different.
pub const FORC: &str = "forc";
pub const FUELUP: &str = "fuelup";
// forc-client is handled differently - its actual binaries are 'forc-run' and 'forc-deploy'
pub const FORC_CLIENT: &str = "forc-client";

const COMPONENTS_TOML: &str = include_str!("../../components.toml");

#[derive(Debug, Deserialize)]
pub struct Components {
    pub component: HashMap<String, Component>,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct Component {
    pub name: String,
    pub is_plugin: Option<bool>,
    pub tarball_prefix: String,
    pub executables: Vec<String>,
    pub repository_name: String,
    pub targets: Vec<String>,
    pub publish: Option<bool>,
    pub show_fuels_version: Option<bool>,
}

impl Component {
    pub fn from_name(name: &str) -> Result<Self> {
        if name == FUELUP {
            return Ok(Component {
                name: FUELUP.to_string(),
                tarball_prefix: FUELUP.to_string(),
                executables: vec![FUELUP.to_string()],
                repository_name: FUELUP.to_string(),
                targets: vec![FUELUP.to_string()],
                is_plugin: Some(false),
                publish: Some(true),
                show_fuels_version: Some(false),
            });
        }

        let components = Components::collect().expect("Could not collect components");

        components
            .component
            .get(name)
            .ok_or_else(|| anyhow!("component with name '{}' does not exist", name))
            .map(|c| c.clone())
    }

    pub fn is_default_forc_plugin(name: &str) -> bool {
        (Self::from_name(FORC)
            .expect("there must always be a `forc` component")
            .executables
            .contains(&name.to_string())
            && name != FORC)
            || name == FORC_CLIENT
    }
}

#[derive(Debug)]
pub struct Plugin {
    pub name: String,
    pub executables: Vec<String>,
    pub publish: Option<bool>,
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

    pub fn collect() -> Result<Components> {
        let components = Self::from_toml(COMPONENTS_TOML)?;
        Ok(components)
    }

    pub fn contains_published(name: &str) -> bool {
        Self::collect_publishables()
            .expect("Failed to collect publishable components")
            .iter()
            .map(|c| c.name.clone())
            .collect::<String>()
            .contains(name)
    }

    pub fn collect_publishables() -> Result<Vec<Component>> {
        let components = Self::from_toml(COMPONENTS_TOML)?;

        let mut publishables: Vec<Component> = components
            .component
            .keys()
            .map(|c| {
                components
                    .component
                    .get(c)
                    .expect("Failed to parse components.toml")
            })
            .filter_map(|c| c.publish.map(|_| c.clone()))
            .collect();

        publishables.sort_by_key(|c| c.name.clone());
        Ok(publishables)
    }

    pub fn collect_exclude_plugins() -> Result<Vec<Component>> {
        let components = Self::from_toml(COMPONENTS_TOML)?;

        let mut main_components: Vec<Component> = components
            .component
            .keys()
            .map(|c| {
                components
                    .component
                    .get(c)
                    .expect("Failed to parse components.toml")
            })
            .filter(|&c| c.is_plugin.is_none())
            .cloned()
            .collect();

        main_components.sort_by_key(|c| c.name.clone());

        Ok(main_components)
    }

    pub fn collect_show_fuels_versions() -> Result<Vec<Component>> {
        let components = Self::from_toml(COMPONENTS_TOML)?;

        let mut components_to_show = components
            .component
            .values()
            .filter_map(|c| {
                if let Some(true) = c.show_fuels_version {
                    Some(c.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<Component>>();

        components_to_show.sort_by_key(|c| c.name.clone());

        Ok(components_to_show)
    }
    pub fn collect_plugins() -> Result<Vec<Plugin>> {
        let components = Self::from_toml(COMPONENTS_TOML)?;

        let mut plugins: Vec<Plugin> = components
            .component
            .keys()
            .map(|c| {
                components
                    .component
                    .get(c)
                    .expect("Failed to parse components.toml")
            })
            .filter(|&c| c.is_plugin.unwrap_or_default())
            .map(|p| Plugin {
                name: p.name.clone(),
                executables: p.executables.clone(),
                publish: p.publish,
            })
            .collect();
        plugins.sort_by_key(|p| p.name.clone());

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

    pub fn is_distributed_by_forc(plugin_name: &str) -> bool {
        let components = Self::from_toml(COMPONENTS_TOML).expect("Failed to parse components toml");
        if let Some(forc) = components.component.get(FORC) {
            return forc.executables.contains(&plugin_name.to_string());
        };

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_toml() -> Result<()> {
        const TOML: &str = r#"
[component.forc-fmt]
name = "forc-fmt"
is_plugin = true
tarball_prefix = "forc-binaries"
executables = ["forc-fmt"]
repository_name = "sway"
targets = ["linux_amd64", "linux_arm64", "darwin_amd64", "darwin_arm64"]
"#;

        let components = Components::from_toml(TOML)?;

        assert_eq!(components.component["forc-fmt"].name, "forc-fmt");
        assert_eq!(components.component["forc-fmt"].is_plugin, Some(true));
        assert_eq!(
            components.component["forc-fmt"].tarball_prefix,
            "forc-binaries"
        );
        assert_eq!(components.component["forc-fmt"].executables, ["forc-fmt"]);
        assert_eq!(components.component["forc-fmt"].repository_name, "sway");
        assert_eq!(
            components.component["forc-fmt"].targets,
            ["linux_amd64", "linux_arm64", "darwin_amd64", "darwin_arm64"]
        );

        Ok(())
    }

    #[test]
    fn test_collect_exclude_plugins() -> Result<()> {
        let components = Components::collect_exclude_plugins().unwrap();
        let actual = components
            .iter()
            .map(|c| c.name.clone())
            .collect::<Vec<String>>();
        let mut expected = ["forc", "fuel-core", "fuel-indexer", "fuel-core-keygen"];
        expected.sort();
        assert_eq!(components.len(), 4);
        assert_eq!(actual, expected);
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
