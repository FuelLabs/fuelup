use anyhow::{bail, Result};
use std::fmt;
use std::fs::{remove_dir_all, remove_file};
use std::path::PathBuf;
use std::str::FromStr;
use tracing::info;

use crate::download::{download_file_and_unpack, link_to_fuelup, unpack_bins, DownloadCfg};
use crate::ops::fuelup_self::self_update;
use crate::path::{
    ensure_dir_exists, fuelup_bin, fuelup_bin_dir, settings_file, toolchain_bin_dir,
};
use crate::settings::SettingsFile;

pub const RESERVED_TOOLCHAIN_NAMES: &[&str] = &["latest", "nightly"];
pub const LATEST: &str = "latest";

pub enum DistToolchainName {
    Latest,
}

impl fmt::Display for DistToolchainName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DistToolchainName::Latest => write!(f, "latest"),
        }
    }
}

impl FromStr for DistToolchainName {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        let name = s.split_once('-').map(|n| n.0);
        match name {
            Some("latest") => Ok(Self::Latest),
            _ => bail!("Unknown name for toolchain: {}", s),
        }
    }
}

pub fn is_official_toolchain(toolchain: &str) -> bool {
    let mut reserved: Vec<String> = Vec::new();
    let triple = TargetTriple::from_host().ok();
    for reserved_toolchain_name in RESERVED_TOOLCHAIN_NAMES {
        reserved.push(reserved_toolchain_name.to_string());

        if triple.is_some() {
            reserved
                .push(reserved_toolchain_name.to_string() + &triple.as_ref().unwrap().to_string());
        }
    }

    reserved.contains(&toolchain.to_string())
}

#[derive(Debug)]
pub struct Toolchain {
    pub name: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TargetTriple(String);

impl fmt::Display for TargetTriple {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl TargetTriple {
    pub fn new(name: &str) -> Self {
        Self(name.to_string())
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

        let target_triple = format!("{}-{}-{}", architecture, vendor, os);

        Ok(Self(target_triple))
    }
}

impl Toolchain {
    pub fn new(name: &str, target: Option<String>) -> Result<Self> {
        let target = match target {
            Some(t) => TargetTriple(t),
            None => TargetTriple::from_host()?,
        };
        let toolchain = format!("{}-{}", name, target);
        let path = toolchain_bin_dir(&toolchain);
        Ok(Self {
            name: toolchain,
            path,
        })
    }

    pub fn from_settings() -> Result<Self> {
        let settings = SettingsFile::new(settings_file());
        let toolchain_name = match settings.with(|s| Ok(s.default_toolchain.clone()))? {
            Some(t) => t,
            None => {
                bail!("No default toolchain detected. Please install or create a toolchain first.")
            }
        };
        let path = toolchain_bin_dir(&toolchain_name);

        Ok(Self {
            name: toolchain_name,
            path,
        })
    }

    pub fn exists(&self) -> bool {
        self.path.exists() && self.path.is_dir()
    }

    pub fn has_component(&self, component: &str) -> bool {
        self.path.join(component).exists()
    }

    pub fn add_component(&self, download_cfg: DownloadCfg) -> Result<DownloadCfg> {
        // Pre-install checks: ensuring toolchain dir, fuelup bin dir, and fuelup exist
        ensure_dir_exists(&self.path)?;

        let fuelup_bin_dir = fuelup_bin_dir();
        ensure_dir_exists(&fuelup_bin_dir)?;

        if !fuelup_bin().is_file() {
            info!("fuelup not found - attempting to self update");
            match self_update() {
                Ok(()) => info!("fuelup installed."),
                Err(e) => bail!("Could not install fuelup: {}", e),
            };
        }

        info!(
            "Installing component {} v{}",
            &download_cfg.name, &download_cfg.version
        );

        if let Err(e) = download_file_and_unpack(&download_cfg, &self.path) {
            bail!(
                "Could not add component {}({}): {}",
                &download_cfg.name,
                &download_cfg.version,
                e
            )
        };

        if let Ok(downloaded) = unpack_bins(&self.path, &fuelup_bin_dir) {
            link_to_fuelup(downloaded)?;
        };

        Ok(download_cfg)
    }

    pub fn remove_component(&self, component: &str) -> Result<()> {
        if self.has_component(component) {
            info!("Removing '{}' from toolchain '{}'", component, self.name);
            let component_path = self.path.join(component);
            remove_file(component_path)?;
            info!("'{}' removed from toolchain '{}'", component, self.name);
        } else {
            info!("'{}' not found in toolchain '{}'", component, self.name);
        }

        Ok(())
    }

    pub fn uninstall_self(&self) -> Result<()> {
        if self.exists() {
            remove_dir_all(self.path.parent().unwrap())?
        }
        Ok(())
    }
}
