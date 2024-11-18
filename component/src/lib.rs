use std::collections::HashMap;

use anyhow::{anyhow, Result};
use serde::Deserialize;
use toml_edit::de;

// Keeping forc since some ways we handle forc is slightly different.
pub const FORC: &str = "forc";
pub const FUELUP: &str = "fuelup";
// forc-client is handled differently - its actual binaries are 'forc-run', 'forc-deploy', and 'forc-submit'
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
            .cloned()
    }

    /// Returns a `Component` from the supplied `Component` name, plugin, or executable
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the component, plugin, or executable
    ///
    /// # Examples
    ///
    /// ```rust
    /// use component::Component;
    ///
    /// let forc = Component::resolve_from_name("forc").unwrap();
    /// let publishable = Component::resolve_from_name("fuel-core").unwrap();
    /// let plugin = Component::resolve_from_name("forc-fmt").unwrap();
    /// let executable = Component::resolve_from_name("forc-run").unwrap();
    /// ```
    pub fn resolve_from_name(name: &str) -> Option<Component> {
        Components::collect().ok().and_then(|components| {
            components.component.get(name).cloned().or_else(|| {
                components
                    .component
                    .values()
                    .find(|comp| comp.executables.contains(&name.to_string()))
                    .cloned()
            })
        })
    }

    pub fn is_default_forc_plugin(name: &str) -> bool {
        (Self::from_name(FORC)
            .expect("there must always be a `forc` component")
            .executables
            .contains(&name.to_string())
            && name != FORC)
            || name == FORC_CLIENT
    }

    /// Tests if the supplied `Component`s come from same distribution
    ///
    /// # Arguments
    ///
    /// * `first` - The first `Component` to compare with
    ///
    /// * `second` - The second `Component` to compare with
    ///
    /// # Examples
    ///
    /// ```rust
    /// use component::Component;
    ///
    /// let forc = Component::from_name("forc").unwrap();
    /// let forc_fmt = Component::from_name("forc-fmt").unwrap();
    ///
    /// assert!(Component::is_in_same_distribution(&forc, &forc_fmt));
    /// ```
    pub fn is_in_same_distribution(first: &Component, second: &Component) -> bool {
        // Components come from the same distribution if:
        //  - their repository names are the same, and
        //  - their tarball prefixes are the same
        first.repository_name == second.repository_name
            && first.tarball_prefix == second.tarball_prefix
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
        let executables = Self::collect_plugins()?
            .iter()
            .flat_map(|p| p.executables.clone())
            .collect();
        Ok(executables)
    }

    /// Tests if the supplied `Component` name, plugin, or executable is
    /// distributed by forc
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the `Component`, plugin, or executable
    ///
    /// # Examples
    ///
    /// ```rust
    /// use component::Components;
    ///
    /// assert!(Components::is_distributed_by_forc("forc"));
    /// assert!(!Components::is_distributed_by_forc("fuel-core"));
    /// assert!(Components::is_distributed_by_forc("forc-fmt"));
    /// assert!(Components::is_distributed_by_forc("forc-run"));
    /// ```
    pub fn is_distributed_by_forc(name: &str) -> bool {
        match name {
            FORC => true,
            _ => Component::from_name(FORC)
                .ok()
                .and_then(|forc| {
                    Component::resolve_from_name(name)
                        .map(|component| Component::is_in_same_distribution(&forc, &component))
                })
                .unwrap_or(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    #[test]
    fn test_toml() -> Result<()> {
        const TOML: &str = indoc! {r#"
            [component.forc-fmt]
            name = "forc-fmt"
            is_plugin = true
            tarball_prefix = "forc-binaries"
            executables = ["forc-fmt"]
            repository_name = "sway"
            targets = ["linux_amd64", "linux_arm64", "darwin_amd64", "darwin_arm64"]
        "#};

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
        let mut expected = ["forc", "fuel-core", "fuel-core-keygen"];
        expected.sort();
        assert_eq!(components.len(), 3);
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

    #[test]
    fn test_from_name_forc() {
        let component = Component::from_name(FORC).unwrap();
        assert_eq!(component.name, FORC, "forc is a publishable component");
    }

    #[test]
    fn test_from_name_publishables() {
        for publishable in Components::collect_publishables().unwrap() {
            let component = Component::from_name(&publishable.name).unwrap();
            assert_eq!(
                component.name, publishable.name,
                "{} is a publishable component",
                publishable.name
            );
        }
    }

    #[test]
    fn test_from_name_plugins() {
        for plugin in Components::collect_plugins().unwrap() {
            let component = Component::from_name(&plugin.name).unwrap();
            assert_eq!(
                component.name, plugin.name,
                "{} is a plugin in {}",
                plugin.name, component.name
            );
        }
    }

    #[test]
    #[should_panic] // This will fail as long as some executables are not plugins
    fn test_from_name_executables() {
        for executable in &Components::collect_plugin_executables().unwrap() {
            let component = Component::from_name(executable).unwrap();
            assert!(
                component.executables.contains(executable),
                "{} is an executable in {}",
                executable,
                component.name
            );
        }
    }

    #[test]
    fn test_resolve_from_name_forc() {
        let component = Component::resolve_from_name(FORC).unwrap();
        assert_eq!(component.name, FORC, "forc is a publishable component");
    }

    #[test]
    fn test_resolve_from_name_publishable() {
        for publishable in Components::collect_publishables().unwrap() {
            let component = Component::resolve_from_name(&publishable.name).unwrap();
            assert_eq!(component.name, publishable.name);
        }
    }

    #[test]
    fn test_resolve_from_name_plugin() {
        for plugin in Components::collect_plugins().unwrap() {
            let component = Component::resolve_from_name(&plugin.name).unwrap();
            assert_eq!(component.name, plugin.name);
        }
    }

    #[test]
    fn test_resolve_from_name_from_executable() {
        let executables = Components::collect_plugin_executables().unwrap();

        for executable in &executables {
            let component = Component::resolve_from_name(executable).unwrap();

            if component.executables.len() == 1 {
                assert_eq!(component.name, *executable);
            } else {
                assert!(component.executables.contains(executable));
            }
        }
    }

    #[test]
    fn test_resolve_from_name_nonexistent() {
        assert!(Component::resolve_from_name("nonexistent-component").is_none());
    }

    #[test]
    fn test_resolve_from_name_case_sensitivity() {
        let original = Component::resolve_from_name("forc");
        let uppercase = Component::resolve_from_name("FORC");
        assert_ne!(original, uppercase);
    }

    #[test]
    fn test_is_distributed_by_forc_forc() {
        assert!(
            Components::is_distributed_by_forc("forc"),
            "forc is distributed by forc"
        );
    }

    #[test]
    fn test_is_distributed_by_forc_publishables() {
        for publishable in Components::collect_publishables().unwrap() {
            let component = Component::from_name(&publishable.name).unwrap();
            is_distributed_by_forc(&component);
        }
    }

    #[test]
    fn test_is_distributed_by_forc_plugins() {
        for plugin in Components::collect_plugins().unwrap() {
            let component = Component::from_name(&plugin.name).unwrap();
            is_distributed_by_forc(&component);
        }
    }

    #[test]
    fn test_is_distributed_by_forc_executables() {
        for executable in Components::collect_plugin_executables().unwrap() {
            let components = Components::collect().unwrap();
            let component = components
                .component
                .values()
                .find(|c| c.executables.contains(&executable))
                .unwrap();

            is_distributed_by_forc(component);
        }
    }

    fn is_distributed_by_forc(component: &Component) {
        let forc = Component::from_name(FORC).unwrap();
        let is_distributed = Components::is_distributed_by_forc(&component.name);

        if Component::is_in_same_distribution(&forc, component) {
            assert!(
                is_distributed,
                "{:?} is distributed by forc",
                component.name
            )
        } else {
            assert!(
                !is_distributed,
                "{:?} is not distributed by forc",
                component.name
            )
        }
    }
}
