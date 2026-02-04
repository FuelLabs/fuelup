use anyhow::{bail, Result};
use component::Component;
use std::fmt;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct TargetTriple(String);

impl fmt::Display for TargetTriple {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl TargetTriple {
    pub fn new(s: &str) -> Result<Self> {
        let Some((architecture, rest)) = s.split_once('-') else {
            bail!("missing vendor-os specifier")
        };
        let Some((vendor, os)) = rest.split_once('-') else {
            bail!("missing os specifier")
        };

        if !["aarch64", "x86_64"].contains(&architecture) {
            bail!("Unsupported architecture: '{}'", architecture);
        }
        if !["apple", "unknown"].contains(&vendor) {
            bail!("Unsupported vendor: '{}'", vendor);
        }
        if !["darwin", "linux-gnu"].contains(&os) {
            bail!("Unsupported os: '{}'", os);
        }
        Ok(Self(s.to_string()))
    }

    pub fn from_host() -> Result<Self> {
        let architecture = match std::env::consts::ARCH {
            "aarch64" | "x86_64" => std::env::consts::ARCH,
            unsupported_arch => bail!("Unsupported architecture: {}", unsupported_arch),
        };
        let vendor = match std::env::consts::OS {
            "macos" => "apple",
            _ => "unknown",
        };
        let os = match std::env::consts::OS {
            "macos" => "darwin",
            "linux" => "linux-gnu",
            unsupported_os => bail!("Unsupported os: {}", unsupported_os),
        };

        let target_triple = format!("{architecture}-{vendor}-{os}");

        Ok(Self(target_triple))
    }

    /// Returns a target triple for the current host from the supplied component name.
    ///
    /// The format is determined by the component's `targets` field in `components.toml`:
    /// - Simplified format: `[darwin|linux]_[arm64|amd64]` (e.g., forc, forc-wallet, forc-crypto)
    /// - Rust triple format: `[arch]-[vendor]-[os]` (e.g., fuel-core, fuel-core-keygen)
    pub fn from_component(name: &str) -> Result<Self> {
        let component = Component::from_name(name)?;
        let uses_simplified_targets = component
            .targets
            .first()
            .map(|t| t.contains('_'))
            .unwrap_or(false);

        if uses_simplified_targets {
            let os = match std::env::consts::OS {
                "macos" => "darwin",
                "linux" => "linux",
                unsupported_os => bail!("Unsupported os: {}", unsupported_os),
            };
            let architecture = match std::env::consts::ARCH {
                "aarch64" => "arm64",
                "x86_64" => "amd64",
                unsupported_arch => bail!("Unsupported architecture: {}", unsupported_arch),
            };
            Ok(Self(format!("{os}_{architecture}")))
        } else {
            let architecture = match std::env::consts::ARCH {
                "aarch64" | "x86_64" => std::env::consts::ARCH,
                unsupported_arch => bail!("Unsupported architecture: {}", unsupported_arch),
            };
            let vendor = match std::env::consts::OS {
                "macos" => "apple",
                _ => "unknown",
            };
            let os = match std::env::consts::OS {
                "macos" => "darwin",
                "linux" => "linux-gnu",
                unsupported_os => bail!("Unsupported os: {}", unsupported_os),
            };
            Ok(Self(format!("{architecture}-{vendor}-{os}")))
        }
    }
}

#[cfg(test)]
mod test_from_component {
    use super::*;
    use component::Components;
    use regex::Regex;

    fn uses_simplified_targets(component: &Component) -> bool {
        component
            .targets
            .first()
            .map(|t| t.contains('_'))
            .unwrap_or(false)
    }

    fn test_target_triple(component: &Component, target_triple: &TargetTriple) {
        let expected_triple_regex = if uses_simplified_targets(component) {
            "^(darwin|linux)_(arm64|amd64)$"
        } else {
            "^(aarch64|x86_64)-(apple|unknown)-(darwin|linux-gnu)$"
        };

        let expected_triple = Regex::new(expected_triple_regex).unwrap();
        assert!(
            expected_triple.is_match(&target_triple.0),
            "{} has triple '{}', expected to match '{}'",
            component.name,
            &target_triple.0,
            expected_triple_regex
        );
    }

    #[test]
    fn all_components() {
        for component in Components::collect().unwrap().component.values() {
            let target_triple = TargetTriple::from_component(&component.name).unwrap();
            test_target_triple(component, &target_triple);
        }
    }

    #[test]
    fn forc_uses_simplified() {
        let target = TargetTriple::from_component("forc").unwrap();
        assert!(
            target.0.contains('_'),
            "forc should use simplified target format"
        );
    }

    #[test]
    fn forc_wallet_uses_simplified() {
        let target = TargetTriple::from_component("forc-wallet").unwrap();
        assert!(
            target.0.contains('_'),
            "forc-wallet should use simplified target format"
        );
    }

    #[test]
    fn forc_crypto_uses_simplified() {
        let target = TargetTriple::from_component("forc-crypto").unwrap();
        assert!(
            target.0.contains('_'),
            "forc-crypto should use simplified target format"
        );
    }

    #[test]
    fn forc_client_uses_simplified() {
        let target = TargetTriple::from_component("forc-client").unwrap();
        assert!(
            target.0.contains('_'),
            "forc-client should use simplified target format"
        );
    }

    #[test]
    fn fuel_core_uses_rust_triple() {
        let target = TargetTriple::from_component("fuel-core").unwrap();
        assert!(
            target.0.contains('-'),
            "fuel-core should use Rust triple format"
        );
    }

    #[test]
    fn fuel_core_keygen_uses_rust_triple() {
        let target = TargetTriple::from_component("fuel-core-keygen").unwrap();
        assert!(
            target.0.contains('-'),
            "fuel-core-keygen should use Rust triple format"
        );
    }
}
