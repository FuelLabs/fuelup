use anyhow::{bail, Context, Result};
use std::fmt;
use std::fs::{remove_dir_all, remove_file};
use std::path::PathBuf;
use std::str::FromStr;
use tracing::info;

use crate::component::SUPPORTED_PLUGINS;
use crate::download::{download_file_and_unpack, link_to_fuelup, unpack_bins, DownloadCfg};
use crate::ops::fuelup_self::self_update;
use crate::path::{
    ensure_dir_exists, fuelup_bin, fuelup_bin_dir, settings_file, toolchain_bin_dir,
};
use crate::settings::SettingsFile;
use crate::target_triple::TargetTriple;
use crate::{channel, component};

pub const RESERVED_TOOLCHAIN_NAMES: &[&str] = &[
    channel::LATEST,
    channel::BETA,
    channel::NIGHTLY,
    channel::STABLE,
];

pub enum DistToolchainName {
    Latest,
}

impl fmt::Display for DistToolchainName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DistToolchainName::Latest => write!(f, "{}", channel::LATEST),
        }
    }
}

impl FromStr for DistToolchainName {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        let name = s.split_once('-').map(|n| n.0);
        match name {
            Some(channel::LATEST) => Ok(Self::Latest),
            _ => bail!("Unknown name for toolchain: {}", s),
        }
    }
}

#[derive(Debug)]
pub struct Toolchain {
    pub name: String,
    pub path: PathBuf,
}

impl Toolchain {
    pub fn new(name: &str) -> Result<Self> {
        let target = TargetTriple::from_host()?;
        let toolchain = format!("{}-{}", name, target);
        let path = toolchain_bin_dir(&toolchain);
        Ok(Self {
            name: toolchain,
            path,
        })
    }

    pub fn from(toolchain: &str) -> Result<Self> {
        let path = toolchain_bin_dir(toolchain);
        Ok(Self {
            name: toolchain.to_string(),
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
            "Adding component {} v{} to '{}'",
            &download_cfg.name, &download_cfg.version, self.name
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
            remove_file(component_path)
                .with_context(|| format!("failed to remove component '{}'", component))?;
            // If component to remove is 'forc', silently remove forc plugins
            if component == component::FORC {
                for component in SUPPORTED_PLUGINS {
                    let component_path = self.path.join(component);
                    remove_file(component_path)
                        .with_context(|| format!("failed to remove component '{}'", component))?;
                }
            }
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
