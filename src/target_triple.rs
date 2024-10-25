use anyhow::{bail, Result};
use component::{self, Component};
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

    pub fn from_component(component: &str) -> Result<Self> {
        match Component::from_name(component).map(|c| c.name)?.as_str() {
            component::FORC => {
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
            }
            _ => {
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
}

#[cfg(test)]
mod test_from_component {
    use super::*;
    use component::{Component, Components};
    use regex::Regex;

    #[test]
    fn forc() {
        let component = Component::from_name("forc").unwrap();
        let target_triple = TargetTriple::from_component(&component.name).unwrap();
        test_target_triple(&component, &target_triple);
    }

    #[test]
    fn publishables() {
        for publishable in Components::collect_publishables().unwrap() {
            let component = Component::from_name(&publishable.name).unwrap();
            let target_triple = TargetTriple::from_component(&component.name).unwrap();
            test_target_triple(&component, &target_triple);
        }
    }

    #[test]
    #[should_panic] // TODO: #654 will fix this
    fn plugins() {
        for plugin in Components::collect_plugins().unwrap() {
            let component = Component::from_name(&plugin.name).unwrap();
            let target_triple = TargetTriple::from_component(&component.name).unwrap();
            test_target_triple(&component, &target_triple);
        }
    }

    #[test]
    #[should_panic] // TODO: #654 will fix this
    fn executables() {
        for executable in Components::collect_plugin_executables().unwrap() {
            let components = Components::collect().unwrap();
            let component = components
                .component
                .values()
                .find(|c| c.executables.contains(&executable))
                .unwrap();

            let target_triple = TargetTriple::from_component(&component.name).unwrap();
            test_target_triple(component, &target_triple);
        }
    }

    fn test_target_triple(component: &Component, target_triple: &TargetTriple) {
        let forc = Component::from_name("forc").unwrap();

        let expected_triple_regex = if Component::is_in_same_distribution(&forc, component) {
            "^(darwin|linux)_(arm64|amd64)$"
        } else {
            "^(aarch64|x86_64)-(apple|unknown)-(darwin|linux-gnu)$"
        };

        let expected_triple = Regex::new(expected_triple_regex).unwrap();
        assert!(
            expected_triple.is_match(&target_triple.0),
            "{} has triple '{}'",
            component.name,
            &target_triple.0
        );
    }
}
