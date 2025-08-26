use crate::{
    channel::{is_dateless_distributed_toolchain, LATEST, NIGHTLY},
    constants::{DATE_FORMAT, FUEL_TOOLCHAIN_TOML_FILE},
    download::DownloadCfg,
    file,
    path::get_fuel_toolchain_toml,
    store::Store,
    target_triple::TargetTriple,
    toolchain::{DistToolchainDescription, Toolchain},
};
use anyhow::{bail, Result};
use semver::Version;
use serde::de::Error;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize};
use std::{collections::HashMap, fmt, path::PathBuf, str::FromStr};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use time::Date;
use toml_edit::{de, ser, value, DocumentMut};
use tracing::{info, warn};

/// Plugin specification supporting both version and path overrides
#[derive(Clone, Debug, Serialize)]
pub enum PluginSpec {
    Version(Version),
    Path(PathBuf),
}

impl<'de> Deserialize<'de> for PluginSpec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        // Try parsing as version first
        if let Ok(version) = Version::parse(&s) {
            return Ok(PluginSpec::Version(version));
        }

        // Otherwise treat as path
        let path = if s.starts_with('/') || s.starts_with("./") || s.starts_with("../") {
            PathBuf::from(s)
        } else if s.contains('/') || s.contains('\\') {
            // Path contains separators but isn't explicitly relative/absolute
            PathBuf::from(s)
        } else {
            // Assume relative path for bare filenames
            PathBuf::from(format!("./{}", s))
        };

        Ok(PluginSpec::Path(path))
    }
}

// For composability with other functionality of fuelup, we want to add
// additional info to OverrideCfg (representation of 'fuel-toolchain.toml').
// In this case, we want the path to the toml file. More info might be
// needed in future.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ToolchainOverride {
    pub cfg: OverrideCfg,
    pub path: PathBuf,
}

// Representation of the entire 'fuel-toolchain.toml'.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OverrideCfg {
    pub toolchain: ToolchainCfg,
    pub components: Option<HashMap<String, Version>>,
    pub plugins: Option<HashMap<String, PluginSpec>>,
}

// Represents the [toolchain] table in 'fuel-toolchain.toml'.
#[derive(Clone, Debug, Deserialize)]
pub struct ToolchainCfg {
    #[serde(deserialize_with = "deserialize_channel")]
    pub channel: Channel,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Channel {
    pub name: String,
    pub date: Option<Date>,
}

impl Serialize for ToolchainCfg {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut c = serializer.serialize_struct("ToolchainCfg", 2)?;
        c.serialize_field("channel", &self.channel.to_string())?;

        c.end()
    }
}

pub fn deserialize_channel<'de, D>(deserializer: D) -> Result<Channel, D::Error>
where
    D: Deserializer<'de>,
{
    let channel_str = String::deserialize(deserializer)?;

    channel_str.parse().map_or_else(
        |_| {
            Err(Error::invalid_value(
                serde::de::Unexpected::Str(&channel_str),
                &"one of <latest-YYYY-MM-DD|nightly-YYYY-MM-DD|testnet|mainnet>",
            ))
        },
        Result::Ok,
    )
}

impl fmt::Display for Channel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.date {
            Some(d) => write!(f, "{}-{}", self.name, d),
            None => write!(f, "{}", self.name),
        }
    }
}

impl FromStr for Channel {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        if is_dateless_distributed_toolchain(s) {
            return Ok(Self {
                name: s.to_string(),
                date: None,
            });
        };

        if let Some((name, d)) = s.split_once('-') {
            Ok(Self {
                name: name.to_string(),
                date: Date::parse(d, DATE_FORMAT).ok(),
            })
        } else {
            if s == LATEST || s == NIGHTLY {
                bail!("'{s}' without date specifier is forbidden");
            }
            bail!("Invalid str for channel: '{}'", s);
        }
    }
}

impl ToolchainOverride {
    // Creates a representation of a 'fuel-toolchain.toml' from a file path.
    // This representation is an OverrideCfg and the file path.
    pub(crate) fn from_path(path: PathBuf) -> Result<Self> {
        let f = file::read_file(FUEL_TOOLCHAIN_TOML_FILE, path.as_path())?;
        let cfg: OverrideCfg = OverrideCfg::from_toml(&f)?;
        Ok(Self { cfg, path })
    }

    #[allow(clippy::indexing_slicing)]
    pub fn to_toml(&self) -> DocumentMut {
        let mut document = toml_edit::DocumentMut::new();

        // Create toolchain table
        let mut toolchain_table = toml_edit::Table::new();
        toolchain_table["channel"] = value(self.cfg.toolchain.channel.to_string());
        document["toolchain"] = toml_edit::Item::Table(toolchain_table);

        // Create components table if present
        if let Some(components) = &self.cfg.components {
            let mut components_table = toml_edit::Table::new();
            for (k, v) in components {
                components_table[k] = value(v.to_string());
            }
            document["components"] = toml_edit::Item::Table(components_table);
        }

        // Create plugins table if present
        if let Some(plugins) = &self.cfg.plugins {
            let mut plugins_table = toml_edit::Table::new();
            for (k, v) in plugins {
                plugins_table[k] = match v {
                    PluginSpec::Version(version) => value(version.to_string()),
                    PluginSpec::Path(path) => value(path.to_string_lossy().to_string()),
                };
            }
            document["plugins"] = toml_edit::Item::Table(plugins_table);
        }

        document
    }

    pub fn from_project_root() -> Option<ToolchainOverride> {
        if let Some(fuel_toolchain_toml_file) = get_fuel_toolchain_toml() {
            match ToolchainOverride::from_path(fuel_toolchain_toml_file) {
                Ok(to) => Some(to),
                Err(e) => {
                    warn!("warning: invalid 'fuel-toolchain.toml' in project root: {e}");
                    None
                }
            }
        } else {
            None
        }
    }

    pub fn get_component_version(&self, component: &str) -> Option<&Version> {
        if let Some(components) = &self.cfg.components {
            components.get(component)
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

    /// Get plugin specification for the given plugin name
    pub fn get_plugin_spec(&self, plugin_name: &str) -> Option<&PluginSpec> {
        self.cfg.plugins.as_ref()?.get(plugin_name)
    }

    /// Resolve plugin path from specification, handling both version and path types
    pub fn resolve_plugin_path(&self, plugin_name: &str) -> Result<Option<PathBuf>> {
        if let Some(spec) = self.get_plugin_spec(plugin_name) {
            match spec {
                PluginSpec::Version(version) => {
                    // Use existing store-based resolution for version specifications
                    let store = Store::from_env()?;
                    let component_name = self.resolve_plugin_component(plugin_name)?;
                    let plugin_path = store
                        .component_dir_path(&component_name, version)
                        .join(plugin_name);

                    // Install component if missing
                    if !store.has_component(&component_name, version) {
                        let target_triple = TargetTriple::from_component(&component_name)?;
                        let download_cfg = DownloadCfg::new(
                            &component_name,
                            target_triple,
                            Some(version.clone()),
                        )?;
                        store.install_component(&download_cfg)?;
                    }

                    Ok(Some(plugin_path))
                }
                PluginSpec::Path(path) => {
                    let resolved_path = if path.is_absolute() {
                        path.clone()
                    } else {
                        // Resolve relative to the fuel-toolchain.toml directory
                        self.path.parent().unwrap_or(&self.path).join(path)
                    };

                    self.validate_plugin_path(&resolved_path, plugin_name)?;
                    Ok(Some(resolved_path))
                }
            }
        } else {
            Ok(None)
        }
    }

    /// Resolve plugin to its component name (similar to existing logic)
    fn resolve_plugin_component(&self, plugin_name: &str) -> Result<String> {
        // Plugins distributed by forc should resolve to 'forc' component
        if component::Components::is_distributed_by_forc(plugin_name) {
            Ok(component::FORC.to_string())
        } else {
            Ok(plugin_name.to_string())
        }
    }

    /// Validate that a plugin path exists and is executable
    fn validate_plugin_path(&self, path: &PathBuf, plugin_name: &str) -> Result<()> {
        if !path.exists() {
            bail!("Plugin path does not exist: {}", path.display());
        }

        if !self.is_executable(path)? {
            bail!("Plugin path is not executable: {}", path.display());
        }

        // Optional: Basic plugin identity validation via --version
        if let Ok(output) = std::process::Command::new(path).arg("--version").output() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let plugin_base = plugin_name.replace('-', "_");
            if !output_str
                .to_lowercase()
                .contains(&plugin_base.to_lowercase())
                && !output_str
                    .to_lowercase()
                    .contains(&plugin_name.to_lowercase())
            {
                warn!(
                    "Plugin at {} may not be '{}' (--version output: {})",
                    path.display(),
                    plugin_name,
                    output_str.trim()
                );
            }
        }

        Ok(())
    }

    /// Check if a file is executable (cross-platform)
    fn is_executable(&self, path: &PathBuf) -> Result<bool> {
        let metadata = std::fs::metadata(path)?;

        #[cfg(unix)]
        {
            Ok(metadata.permissions().mode() & 0o111 != 0)
        }

        #[cfg(windows)]
        {
            // On Windows, check if it's a file and has an executable extension
            Ok(metadata.is_file()
                && path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| matches!(ext.to_lowercase().as_str(), "exe" | "bat" | "cmd"))
                    .unwrap_or(false))
        }
    }
}

impl OverrideCfg {
    pub fn new(toolchain: ToolchainCfg, components: Option<HashMap<String, Version>>) -> Self {
        Self {
            toolchain,
            components,
            plugins: None,
        }
    }

    // Creates a representation of a 'fuel-toolchain.toml' from a toml string.
    // This is used in the implementation of ToolchainOverride, which is just
    // an OverrideCfg with its file path.
    pub(crate) fn from_toml(toml: &str) -> Result<Self> {
        let cfg: OverrideCfg = de::from_str(toml)?;
        if DistToolchainDescription::from_str(&cfg.toolchain.channel.to_string()).is_err() {
            bail!("Invalid channel '{}'", &cfg.toolchain.channel)
        }

        if let Some(components) = cfg.components.as_ref() {
            if components.is_empty() {
                bail!("'[components]' table is declared with no components")
            }
        }

        if let Some(plugins) = cfg.plugins.as_ref() {
            if plugins.is_empty() {
                bail!("'[plugins]' table is declared with no plugins")
            }
        }

        Ok(cfg)
    }

    pub fn to_string_pretty(self) -> Result<String, ser::Error> {
        ser::to_string_pretty(&self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channel::{MAINNET, NIGHTLY, TESTNET};
    use indoc::indoc;

    #[test]
    fn parse_toolchain_override_latest_with_date() {
        const TOML: &str = indoc! {r#"
            [toolchain]
            channel = "latest-2023-01-09"
        "#};
        let cfg = OverrideCfg::from_toml(TOML).unwrap();
        assert_eq!(cfg.toolchain.channel.to_string(), "latest-2023-01-09");
        assert!(cfg.components.is_none());
        assert_eq!(TOML, cfg.to_string_pretty().unwrap());
    }

    #[test]
    fn parse_toolchain_override_nightly_with_date() {
        const TOML: &str = indoc! {r#"
            [toolchain]
            channel = "nightly-2023-01-09"

            [components]
            forc = "0.33.0"
        "#};
        let cfg = OverrideCfg::from_toml(TOML).unwrap();
        assert_eq!(cfg.toolchain.channel.to_string(), "nightly-2023-01-09");
        assert_eq!(
            cfg.components.as_ref().unwrap().get("forc").unwrap(),
            &Version::new(0, 33, 0)
        );
        assert_eq!(TOML, cfg.to_string_pretty().unwrap());
    }

    #[test]
    fn parse_toolchain_override_channel_without_date_error() {
        const LATEST: &str = indoc! {r#"
            [toolchain]
            channel = "latest"
        "#};
        const NIGHTLY: &str = indoc! {r#"
            [toolchain]
            channel = "nightly"
        "#};

        let result = OverrideCfg::from_toml(LATEST);
        assert!(result.is_err());
        let e = result.unwrap_err();
        assert!(e
            .to_string().contains(
            "invalid value: string \"latest\", expected one of <latest-YYYY-MM-DD|nightly-YYYY-MM-DD|testnet|mainnet>"));

        let result = OverrideCfg::from_toml(NIGHTLY);
        assert!(result.is_err());
        let e = result.unwrap_err();

        assert!(e
            .to_string().contains(
            "invalid value: string \"nightly\", expected one of <latest-YYYY-MM-DD|nightly-YYYY-MM-DD|testnet|mainnet>"));
    }

    #[test]
    fn parse_toolchain_override_invalid_tomls() {
        const EMPTY_STR: &str = "";
        const EMPTY_TOOLCHAIN: &str = indoc! {r#"
            [toolchain]
        "#};
        const INVALID_CHANNEL: &str = indoc! {r#"
            [toolchain]
            channel = "invalid-channel"
        "#};
        const EMPTY_COMPONENTS: &str = indoc! {r#"
            [toolchain]
            channel = "testnet"

            [components]
        "#};

        for toml in [
            EMPTY_STR,
            EMPTY_TOOLCHAIN,
            INVALID_CHANNEL,
            EMPTY_COMPONENTS,
        ] {
            assert!(OverrideCfg::from_toml(toml).is_err());
        }
    }

    #[test]
    fn channel_from_str() {
        assert!(Channel::from_str(TESTNET).is_ok());
        assert!(Channel::from_str(MAINNET).is_ok());
        assert!(Channel::from_str(NIGHTLY).is_err());
        assert!(Channel::from_str(LATEST).is_err());
    }

    #[test]
    fn parse_plugin_override_version() {
        const TOML: &str = indoc! {r#"
            [toolchain]
            channel = "testnet"

            [plugins]
            forc-doc = "0.68.1"
        "#};
        let cfg = OverrideCfg::from_toml(TOML).unwrap();
        assert_eq!(cfg.toolchain.channel.to_string(), "testnet");

        let plugins = cfg.plugins.as_ref().unwrap();
        let forc_doc_spec = plugins.get("forc-doc").unwrap();

        match forc_doc_spec {
            PluginSpec::Version(v) => assert_eq!(v, &Version::new(0, 68, 1)),
            PluginSpec::Path(_) => panic!("Expected version, got path"),
        }
    }

    #[test]
    fn parse_plugin_override_path() {
        const TOML: &str = indoc! {r#"
            [toolchain]
            channel = "testnet"

            [plugins]
            forc-lsp = "/path/to/local/forc-lsp"
            forc-fmt = "./local/forc-fmt"
        "#};
        let cfg = OverrideCfg::from_toml(TOML).unwrap();
        assert_eq!(cfg.toolchain.channel.to_string(), "testnet");

        let plugins = cfg.plugins.as_ref().unwrap();

        // Test absolute path
        let forc_lsp_spec = plugins.get("forc-lsp").unwrap();
        match forc_lsp_spec {
            PluginSpec::Path(path) => assert_eq!(path, &PathBuf::from("/path/to/local/forc-lsp")),
            PluginSpec::Version(_) => panic!("Expected path, got version"),
        }

        // Test relative path
        let forc_fmt_spec = plugins.get("forc-fmt").unwrap();
        match forc_fmt_spec {
            PluginSpec::Path(path) => assert_eq!(path, &PathBuf::from("./local/forc-fmt")),
            PluginSpec::Version(_) => panic!("Expected path, got version"),
        }
    }

    #[test]
    fn parse_plugin_override_mixed() {
        const TOML: &str = indoc! {r#"
            [toolchain]
            channel = "mainnet"

            [components]
            forc = "0.67.2"

            [plugins]
            forc-doc = "0.68.1"
            forc-lsp = "/custom/forc-lsp"
            forc-fmt = "./dev/forc-fmt"
        "#};
        let cfg = OverrideCfg::from_toml(TOML).unwrap();

        // Verify all sections are parsed correctly
        assert_eq!(cfg.toolchain.channel.to_string(), "mainnet");
        assert_eq!(
            cfg.components.as_ref().unwrap().get("forc").unwrap(),
            &Version::new(0, 67, 2)
        );

        let plugins = cfg.plugins.as_ref().unwrap();
        assert_eq!(plugins.len(), 3);

        // Check version plugin
        match plugins.get("forc-doc").unwrap() {
            PluginSpec::Version(v) => assert_eq!(v, &Version::new(0, 68, 1)),
            _ => panic!("Expected version"),
        }

        // Check path plugins
        match plugins.get("forc-lsp").unwrap() {
            PluginSpec::Path(path) => assert_eq!(path, &PathBuf::from("/custom/forc-lsp")),
            _ => panic!("Expected path"),
        }

        match plugins.get("forc-fmt").unwrap() {
            PluginSpec::Path(path) => assert_eq!(path, &PathBuf::from("./dev/forc-fmt")),
            _ => panic!("Expected path"),
        }
    }

    #[test]
    fn parse_plugin_spec_auto_detection() {
        // Test version string detection
        let version_spec: PluginSpec = serde_json::from_str("\"1.2.3\"").unwrap();
        match version_spec {
            PluginSpec::Version(v) => assert_eq!(v, Version::new(1, 2, 3)),
            _ => panic!("Expected version"),
        }

        // Test path detection - absolute
        let abs_path_spec: PluginSpec = serde_json::from_str("\"/usr/bin/forc-lsp\"").unwrap();
        match abs_path_spec {
            PluginSpec::Path(path) => assert_eq!(path, PathBuf::from("/usr/bin/forc-lsp")),
            _ => panic!("Expected path"),
        }

        // Test path detection - relative
        let rel_path_spec: PluginSpec = serde_json::from_str("\"./bin/forc-fmt\"").unwrap();
        match rel_path_spec {
            PluginSpec::Path(path) => assert_eq!(path, PathBuf::from("./bin/forc-fmt")),
            _ => panic!("Expected path"),
        }

        // Test bare filename gets converted to relative path
        let bare_name_spec: PluginSpec = serde_json::from_str("\"forc-custom\"").unwrap();
        match bare_name_spec {
            PluginSpec::Path(path) => assert_eq!(path, PathBuf::from("./forc-custom")),
            _ => panic!("Expected path"),
        }
    }

    #[test]
    fn parse_toolchain_override_invalid_plugins() {
        const EMPTY_PLUGINS: &str = indoc! {r#"
            [toolchain]
            channel = "testnet"

            [plugins]
        "#};

        let result = OverrideCfg::from_toml(EMPTY_PLUGINS);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("'[plugins]' table is declared with no plugins"));
    }

    #[test]
    fn serialize_plugin_overrides() {
        use std::collections::HashMap;
        use tempfile::tempdir;

        let mut components = HashMap::new();
        components.insert("forc".to_string(), Version::new(0, 67, 2));

        let mut plugins = HashMap::new();
        plugins.insert(
            "forc-doc".to_string(),
            PluginSpec::Version(Version::new(0, 68, 1)),
        );
        plugins.insert(
            "forc-lsp".to_string(),
            PluginSpec::Path(PathBuf::from("./custom/forc-lsp")),
        );

        let cfg = OverrideCfg {
            toolchain: ToolchainCfg {
                channel: Channel {
                    name: "mainnet".to_string(),
                    date: None,
                },
            },
            components: Some(components),
            plugins: Some(plugins),
        };

        // Create a temporary ToolchainOverride to test the to_toml() method
        let temp_dir = tempdir().unwrap();
        let toml_path = temp_dir.path().join("fuel-toolchain.toml");

        let toolchain_override = ToolchainOverride {
            cfg,
            path: toml_path,
        };

        let toml_doc = toolchain_override.to_toml();
        let toml_str = toml_doc.to_string();

        // Verify the serialized TOML contains all expected sections
        assert!(toml_str.contains("[toolchain]"));
        assert!(toml_str.contains("channel = \"mainnet\""));
        assert!(toml_str.contains("[components]"));
        assert!(toml_str.contains("forc = \"0.67.2\""));
        assert!(toml_str.contains("[plugins]"));
        assert!(toml_str.contains("forc-doc = \"0.68.1\""));
        assert!(toml_str.contains("forc-lsp = \"./custom/forc-lsp\""));
    }
}
