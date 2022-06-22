use anyhow::{bail, Result};
use std::fmt;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;
use tracing::info;

use crate::download::{download_file_and_unpack, link_to_fuelup, unpack_bins, DownloadCfg};
use crate::ops::fuelup_self::self_update;
use crate::path::{ensure_dir_exists, fuelup_bin, fuelup_bin_dir, toolchain_bin_dir};

pub enum ToolchainName {
    Latest,
}

impl FromStr for ToolchainName {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "latest" => Ok(Self::Latest),
            _ => bail!("Unknown name for toolchain: {}", s),
        }
    }
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
        let name = match ToolchainName::from_str(name)? {
            ToolchainName::Latest => format!("{}-{}", name, target),
        };
        let path = toolchain_bin_dir(&name);
        Ok(Self { name, path })
    }

    pub fn from(name: &str) -> Self {
        let path = toolchain_bin_dir(name);
        Self {
            name: name.to_string(),
            path,
        }
    }

    pub fn from_settings(toolchain: &str) -> Result<Self> {
        let split = toolchain.split_once('-').unwrap();
        let name = split.0.to_string();
        let target = TargetTriple(split.1.to_string());
        let path = match ToolchainName::from_str(&name)? {
            ToolchainName::Latest => toolchain_bin_dir(&format!("{}-{}", name, target)),
        };
        Ok(Self { name, path })
    }

    pub fn from_path(&self, path: &Path) -> Result<Self> {
        let name = path.file_name().unwrap();

        // Minimally check that there's a /bin directory
        if !path.join("bin").is_dir() {
            bail!("Invalid toolchain path");
        }

        Ok(Self {
            name: name.to_string_lossy().to_string(),
            path: path.to_path_buf(),
        })
    }

    pub fn add_component(&self, download_cfg: DownloadCfg) -> Result<DownloadCfg> {
        // First ensure that toolchain path exists
        ensure_dir_exists(&self.path);

        // Then, ensure that fuelup bin dir exists
        let fuelup_bin_dir = fuelup_bin_dir();
        ensure_dir_exists(&fuelup_bin_dir);

        // Ensure that fuelup exists under $HOME/.fuelup/bin
        if !fuelup_bin().is_file() {
            info!("fuelup not found - attempting to self update");
            match self_update() {
                Ok(()) => info!("fuelup installed."),
                Err(e) => bail!("Could not install fuelup: {}", e),
            };
        }

        info!("Fetching {} {}", &download_cfg.name, &download_cfg.version);

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
}
