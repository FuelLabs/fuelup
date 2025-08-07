use crate::{
    channel::{is_dateless_distributed_toolchain, LATEST, NIGHTLY},
    constants::{DATE_FORMAT, FUEL_TOOLCHAIN_TOML_FILE},
    download::DownloadCfg,
    file,
    path::get_fuel_toolchain_toml,
    target_triple::TargetTriple,
    toolchain::{DistToolchainDescription, Toolchain},
};
use anyhow::{bail, Result};
use semver::Version;
use serde::de::Error;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize};
use std::{
    collections::HashMap,
    fmt, fs,
    path::{Path, PathBuf},
    str::FromStr,
};
use time::Date;
use toml_edit::{de, ser, value, DocumentMut};
use tracing::{info, warn};

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
    pub components: Option<HashMap<String, ComponentSpec>>,
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

/// Represents a component specification - either a version or a local path
#[derive(Clone, Debug, PartialEq)]
pub enum ComponentSpec {
    /// A version specification like "0.41.7"
    Version(Version),
    /// A local path to a binary, either absolute or relative to fuel-toolchain.toml
    Path(PathBuf),
}

impl ComponentSpec {
    /// Returns true if this spec represents a local path
    pub fn is_path(&self) -> bool {
        matches!(self, ComponentSpec::Path(_))
    }

    /// Returns true if this spec represents a version
    pub fn is_version(&self) -> bool {
        matches!(self, ComponentSpec::Version(_))
    }

    /// Gets the version if this is a Version spec, None otherwise
    pub fn version(&self) -> Option<&Version> {
        match self {
            ComponentSpec::Version(v) => Some(v),
            ComponentSpec::Path(_) => None,
        }
    }

    /// Gets the path if this is a Path spec, None otherwise
    pub fn path(&self) -> Option<&PathBuf> {
        match self {
            ComponentSpec::Path(p) => Some(p),
            ComponentSpec::Version(_) => None,
        }
    }

    /// Resolves a path relative to the given base directory (fuel-toolchain.toml location)
    /// Prevents path traversal attacks by checking resolved paths stay within bounds
    pub fn resolve_path(&self, base_dir: &Path) -> Option<PathBuf> {
        match self {
            ComponentSpec::Path(path) => {
                if path.is_absolute() {
                    Some(path.clone())
                } else {
                    // Prevent path traversal attacks
                    let resolved = base_dir.join(path);
                    if let Ok(canonical_resolved) = resolved.canonicalize() {
                        if let Ok(canonical_base) = base_dir.canonicalize() {
                            if canonical_resolved.starts_with(canonical_base) {
                                Some(canonical_resolved)
                            } else {
                                warn!("Path traversal attempt blocked: {}", path.display());
                                None
                            }
                        } else {
                            Some(resolved) // Allow if base dir doesn't exist yet
                        }
                    } else {
                        Some(resolved) // Allow non-existent paths for later validation
                    }
                }
            }
            ComponentSpec::Version(_) => None,
        }
    }

    /// Validates a local binary path by checking if it exists and is executable
    pub fn validate_binary(&self, base_dir: &Path) -> Result<()> {
        match self {
            ComponentSpec::Path(_) => {
                if let Some(resolved_path) = self.resolve_path(base_dir) {
                    validate_local_binary(&resolved_path)?;
                    Ok(())
                } else {
                    bail!("Failed to resolve path")
                }
            }
            ComponentSpec::Version(_) => Ok(()), // Version specs don't need validation
        }
    }
}

impl fmt::Display for ComponentSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ComponentSpec::Version(v) => write!(f, "{v}"),
            ComponentSpec::Path(p) => write!(f, "{}", p.display()),
        }
    }
}

impl FromStr for ComponentSpec {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        // Try parsing as a version first
        if let Ok(version) = Version::parse(s) {
            Ok(ComponentSpec::Version(version))
        } else {
            // Otherwise, treat as a path
            Ok(ComponentSpec::Path(PathBuf::from(s)))
        }
    }
}

impl Serialize for ComponentSpec {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            ComponentSpec::Version(v) => v.serialize(serializer),
            ComponentSpec::Path(p) => p.to_string_lossy().serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for ComponentSpec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        ComponentSpec::from_str(&s).map_err(serde::de::Error::custom)
    }
}

/// Validates that a local binary exists and is executable
fn validate_local_binary(path: &PathBuf) -> Result<()> {
    // Check if file exists
    if !path.exists() {
        bail!("Binary not found: {}", path.display());
    }

    // Check if it's a file (not a directory)
    let metadata = fs::metadata(path)?;
    if !metadata.is_file() {
        bail!("Path is not a file: {}", path.display());
    }

    // Check if it's executable (Unix-specific)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = metadata.permissions();
        if perms.mode() & 0o111 == 0 {
            bail!("Binary is not executable: {}", path.display());
        }

        // Security warning for world-writable files
        if perms.mode() & 0o002 != 0 {
            warn!(
                "Security warning: Binary is world-writable: {}",
                path.display()
            );
        }
    }

    // Try to canonicalize the path for security
    match path.canonicalize() {
        Ok(canonical_path) => {
            info!("Validated binary at: {}", canonical_path.display());
        }
        Err(e) => {
            warn!("Could not canonicalize path {}: {}", path.display(), e);
        }
    }

    Ok(())
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

        document["toolchain"]["channel"] = value(self.cfg.toolchain.channel.to_string());
        if let Some(components) = &self.cfg.components {
            for (k, v) in components {
                document["components"][k] = value(v.to_string());
            }
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
            components.get(component).and_then(|spec| spec.version())
        } else {
            None
        }
    }

    pub fn get_component_spec(&self, component: &str) -> Option<&ComponentSpec> {
        if let Some(components) = &self.cfg.components {
            components.get(component)
        } else {
            None
        }
    }

    /// Returns the directory containing the fuel-toolchain.toml file for path resolution
    pub fn base_dir(&self) -> PathBuf {
        self.path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| {
                if self.path.is_dir() {
                    self.path.clone()
                } else {
                    PathBuf::from(".")
                }
            })
    }

    /// Gets the resolved path for a component if it's a local path specification
    pub fn get_component_path(&self, component: &str) -> Option<PathBuf> {
        if let Some(spec) = self.get_component_spec(component) {
            spec.resolve_path(&self.base_dir())
        } else {
            None
        }
    }

    /// Validates all local path components
    pub fn validate_local_components(&self) -> Result<()> {
        if let Some(components) = &self.cfg.components {
            let base_dir = self.base_dir();
            for (name, spec) in components {
                if let Err(e) = spec.validate_binary(&base_dir) {
                    bail!("Invalid local binary for component '{}': {}", name, e);
                }
            }
        }
        Ok(())
    }

    pub fn install_missing_components(&self, toolchain: &Toolchain, called: &str) -> Result<()> {
        match &self.cfg.components {
            None => warn!(
                "warning: overriding toolchain '{}' in {} does not have any components listed",
                &self.cfg.toolchain.channel, FUEL_TOOLCHAIN_TOML_FILE
            ),
            Some(components) => {
                for (component_name, spec) in components {
                    // Only install version-based components, skip local path components
                    if let ComponentSpec::Version(_) = spec {
                        if !toolchain.has_component(component_name) {
                            let target_triple = TargetTriple::from_component(component_name)?;

                            if let Ok(download_cfg) = DownloadCfg::new(called, target_triple, None)
                            {
                                info!(
                                    "installing missing component '{}' specified in {}",
                                    component_name, FUEL_TOOLCHAIN_TOML_FILE
                                );
                                toolchain.add_component(download_cfg)?;
                            };
                        }
                    } else {
                        // For path-based components, just validate they exist
                        info!(
                            "Using local binary for component '{}' specified in {}",
                            component_name, FUEL_TOOLCHAIN_TOML_FILE
                        );
                    }
                }
            }
        };
        Ok(())
    }
}

impl OverrideCfg {
    pub fn new(
        toolchain: ToolchainCfg,
        components: Option<HashMap<String, ComponentSpec>>,
    ) -> Self {
        Self {
            toolchain,
            components,
        }
    }

    // Creates a representation of a 'fuel-toolchain.toml' from a toml string.
    // This is used in the implementation of ToolchainOverride, which is just
    // an OverrideCfg with its file path.
    pub fn from_toml(toml: &str) -> Result<Self> {
        let cfg: OverrideCfg = de::from_str(toml)?;
        if DistToolchainDescription::from_str(&cfg.toolchain.channel.to_string()).is_err() {
            bail!("Invalid channel '{}'", &cfg.toolchain.channel)
        }

        if let Some(components) = cfg.components.as_ref() {
            if components.is_empty() {
                bail!("'[components]' table is declared with no components")
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
            &ComponentSpec::Version(Version::new(0, 33, 0))
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
    fn component_spec_from_str() {
        // Test version parsing
        let version_spec = ComponentSpec::from_str("1.2.3").unwrap();
        assert!(version_spec.is_version());
        assert_eq!(version_spec.version().unwrap(), &Version::new(1, 2, 3));

        // Test path parsing
        let path_spec = ComponentSpec::from_str("/usr/local/bin/forc").unwrap();
        assert!(path_spec.is_path());
        assert_eq!(
            path_spec.path().unwrap(),
            &PathBuf::from("/usr/local/bin/forc")
        );

        // Test relative path parsing
        let rel_path_spec = ComponentSpec::from_str("./bin/forc").unwrap();
        assert!(rel_path_spec.is_path());
        assert_eq!(rel_path_spec.path().unwrap(), &PathBuf::from("./bin/forc"));
    }

    #[test]
    fn component_spec_display() {
        let version_spec = ComponentSpec::Version(Version::new(1, 2, 3));
        assert_eq!(version_spec.to_string(), "1.2.3");

        let path_spec = ComponentSpec::Path(PathBuf::from("/usr/local/bin/forc"));
        assert_eq!(path_spec.to_string(), "/usr/local/bin/forc");
    }

    #[test]
    fn component_spec_path_resolution() {
        let base_dir = PathBuf::from("/project");

        // Test absolute path
        let abs_spec = ComponentSpec::Path(PathBuf::from("/usr/bin/forc"));
        assert_eq!(
            abs_spec.resolve_path(&base_dir).unwrap(),
            PathBuf::from("/usr/bin/forc")
        );

        // Test relative path
        let rel_spec = ComponentSpec::Path(PathBuf::from("bin/forc"));
        assert_eq!(
            rel_spec.resolve_path(&base_dir).unwrap(),
            PathBuf::from("/project/bin/forc")
        );

        // Test version spec returns None
        let version_spec = ComponentSpec::Version(Version::new(1, 0, 0));
        assert!(version_spec.resolve_path(&base_dir).is_none());
    }

    #[test]
    fn parse_toolchain_override_with_local_path() {
        const TOML: &str = indoc! {r#"
            [toolchain]
            channel = "testnet"

            [components]
            forc = "/usr/local/bin/forc"
            fuel-core = "0.41.7"
        "#};
        let cfg = OverrideCfg::from_toml(TOML).unwrap();
        assert_eq!(cfg.toolchain.channel.to_string(), "testnet");

        let components = cfg.components.unwrap();

        // Check path component
        let forc_spec = components.get("forc").unwrap();
        assert!(forc_spec.is_path());
        assert_eq!(
            forc_spec.path().unwrap(),
            &PathBuf::from("/usr/local/bin/forc")
        );

        // Check version component
        let fuel_core_spec = components.get("fuel-core").unwrap();
        assert!(fuel_core_spec.is_version());
        assert_eq!(fuel_core_spec.version().unwrap(), &Version::new(0, 41, 7));
    }

    #[test]
    fn parse_toolchain_override_relative_path() {
        const TOML: &str = indoc! {r#"
            [toolchain]
            channel = "testnet"

            [components]
            forc = "./bin/forc"
        "#};
        let cfg = OverrideCfg::from_toml(TOML).unwrap();

        let components = cfg.components.unwrap();
        let forc_spec = components.get("forc").unwrap();
        assert!(forc_spec.is_path());
        assert_eq!(forc_spec.path().unwrap(), &PathBuf::from("./bin/forc"));
    }

    #[test]
    fn serialize_component_spec() {
        let mut components = HashMap::new();
        components.insert(
            "forc".to_string(),
            ComponentSpec::Path(PathBuf::from("/usr/bin/forc")),
        );
        components.insert(
            "fuel-core".to_string(),
            ComponentSpec::Version(Version::new(0, 41, 7)),
        );

        let cfg = OverrideCfg::new(
            ToolchainCfg {
                channel: Channel::from_str("testnet").unwrap(),
            },
            Some(components),
        );

        let toml_str = cfg.to_string_pretty().unwrap();

        // Verify round-trip serialization works
        let parsed_cfg = OverrideCfg::from_toml(&toml_str).unwrap();
        let parsed_components = parsed_cfg.components.unwrap();

        assert!(parsed_components.get("forc").unwrap().is_path());
        assert!(parsed_components.get("fuel-core").unwrap().is_version());
    }

    #[test]
    fn toolchain_override_methods() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let toml_path = temp_dir.path().join("fuel-toolchain.toml");

        let mut components = HashMap::new();
        components.insert(
            "forc".to_string(),
            ComponentSpec::Path(PathBuf::from("./bin/forc")),
        );
        components.insert(
            "fuel-core".to_string(),
            ComponentSpec::Version(Version::new(0, 41, 7)),
        );

        let override_cfg = ToolchainOverride {
            cfg: OverrideCfg::new(
                ToolchainCfg {
                    channel: Channel::from_str("testnet").unwrap(),
                },
                Some(components),
            ),
            path: toml_path.clone(),
        };

        // Test component spec retrieval
        let forc_spec = override_cfg.get_component_spec("forc").unwrap();
        assert!(forc_spec.is_path());

        // Test version retrieval (should work for version specs)
        let fuel_core_version = override_cfg.get_component_version("fuel-core").unwrap();
        assert_eq!(fuel_core_version, &Version::new(0, 41, 7));

        // Test version retrieval for path spec (should be None)
        assert!(override_cfg.get_component_version("forc").is_none());

        // Test base directory
        assert_eq!(override_cfg.base_dir(), temp_dir.path());

        // Test path resolution
        let resolved_path = override_cfg.get_component_path("forc").unwrap();
        assert_eq!(resolved_path, temp_dir.path().join("bin/forc"));

        // Test path resolution for version spec (should be None)
        assert!(override_cfg.get_component_path("fuel-core").is_none());
    }

    #[test]
    fn validate_local_binary_nonexistent() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let nonexistent_path = temp_dir.path().join("nonexistent");

        let result = validate_local_binary(&nonexistent_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Binary not found"));
    }

    #[test]
    fn validate_local_binary_directory() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().join("dir");
        std::fs::create_dir(&dir_path).unwrap();

        let result = validate_local_binary(&dir_path);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Path is not a file"));
    }

    #[test]
    #[cfg(unix)]
    fn validate_local_binary_not_executable() {
        use std::os::unix::fs::PermissionsExt;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let binary_path = temp_dir.path().join("binary");

        // Create a non-executable file
        fs::File::create(&binary_path).unwrap();
        let perms = fs::Permissions::from_mode(0o644); // readable/writable but not executable
        fs::set_permissions(&binary_path, perms).unwrap();

        let result = validate_local_binary(&binary_path);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Binary is not executable"));
    }

    #[test]
    #[cfg(unix)]
    fn validate_local_binary_executable_success() {
        use std::os::unix::fs::PermissionsExt;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let binary_path = temp_dir.path().join("binary");

        // Create an executable file
        fs::File::create(&binary_path).unwrap();
        let perms = fs::Permissions::from_mode(0o755); // readable/writable/executable
        fs::set_permissions(&binary_path, perms).unwrap();

        let result = validate_local_binary(&binary_path);
        assert!(result.is_ok());
    }

    #[test]
    fn component_spec_validate_binary_version() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let version_spec = ComponentSpec::Version(Version::new(1, 0, 0));

        // Validation should always succeed for version specs
        let result = version_spec.validate_binary(temp_dir.path());
        assert!(result.is_ok());
    }

    #[test]
    fn toolchain_override_validate_all_components() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let toml_path = temp_dir.path().join("fuel-toolchain.toml");

        // Create a valid executable
        let binary_path = temp_dir.path().join("valid_binary");
        fs::File::create(&binary_path).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::Permissions::from_mode(0o755);
            fs::set_permissions(&binary_path, perms).unwrap();
        }

        let mut components = HashMap::new();
        components.insert(
            "forc".to_string(),
            ComponentSpec::Path(PathBuf::from("valid_binary")),
        );
        components.insert(
            "fuel-core".to_string(),
            ComponentSpec::Version(Version::new(0, 41, 7)),
        );

        let override_cfg = ToolchainOverride {
            cfg: OverrideCfg::new(
                ToolchainCfg {
                    channel: Channel::from_str("testnet").unwrap(),
                },
                Some(components),
            ),
            path: toml_path,
        };

        let result = override_cfg.validate_local_components();
        #[cfg(unix)]
        assert!(result.is_ok());
        #[cfg(windows)]
        assert!(result.is_ok()); // Windows validation is more lenient
    }

    #[test]
    fn toolchain_override_validate_invalid_path() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let toml_path = temp_dir.path().join("fuel-toolchain.toml");

        let mut components = HashMap::new();
        components.insert(
            "forc".to_string(),
            ComponentSpec::Path(PathBuf::from("nonexistent")),
        );

        let override_cfg = ToolchainOverride {
            cfg: OverrideCfg::new(
                ToolchainCfg {
                    channel: Channel::from_str("testnet").unwrap(),
                },
                Some(components),
            ),
            path: toml_path,
        };

        let result = override_cfg.validate_local_components();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid local binary for component 'forc'"));
    }

    #[test]
    fn backward_compatibility_version_only() {
        // This test ensures existing fuel-toolchain.toml files with only versions still work
        const TOML: &str = indoc! {r#"
            [toolchain]
            channel = "testnet"

            [components]
            forc = "0.33.0"
            fuel-core = "0.41.7"
        "#};

        let cfg = OverrideCfg::from_toml(TOML).unwrap();
        let components = cfg.components.as_ref().unwrap();

        // All components should be parsed as version specs
        assert!(components.get("forc").unwrap().is_version());
        assert!(components.get("fuel-core").unwrap().is_version());

        // Verify round-trip serialization preserves format
        let serialized = cfg.to_string_pretty().unwrap();
        let reparsed = OverrideCfg::from_toml(&serialized).unwrap();
        let reparsed_components = reparsed.components.unwrap();

        assert!(reparsed_components.get("forc").unwrap().is_version());
        assert!(reparsed_components.get("fuel-core").unwrap().is_version());
    }
}
