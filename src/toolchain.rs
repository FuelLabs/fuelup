use anyhow::{bail, Result};
use std::fmt;
use std::str::FromStr;
use std::{fs, path::PathBuf};
use tracing::info;

use crate::download::{download_file_and_unpack, unpack_and_link_bins, DownloadCfg};
use crate::path::{fuelup_bin_dir, toolchain_bin_dir};

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
    pub target: TargetTriple,
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
        let path = match ToolchainName::from_str(name)? {
            ToolchainName::Latest => toolchain_bin_dir(&format!("{}-{}", name, target)),
        };
        Ok(Self {
            name: name.to_string(),
            path,
            target,
        })
    }

    pub fn from_settings(toolchain: &str) -> Result<Self> {
        let split = toolchain.split_once('-').unwrap();
        let name = split.0.to_string();
        let target = TargetTriple(split.1.to_string());
        let path = match ToolchainName::from_str(&name)? {
            ToolchainName::Latest => toolchain_bin_dir(&format!("{}-{}", name, target)),
        };
        Ok(Self { name, path, target })
    }

    pub fn add_component(&self, download_cfg: DownloadCfg) -> Result<DownloadCfg> {
        if !self.path.is_dir() {
            fs::create_dir_all(&self.path).expect("Unable to create fuelup directory");
        }

        info!("Fetching {} {}", &download_cfg.name, &download_cfg.version);

        if download_file_and_unpack(&download_cfg, &self.path).is_err() {
            bail!("{} {}", &download_cfg.name, &download_cfg.version)
        };

        if unpack_and_link_bins(&self.path, &fuelup_bin_dir()).is_err() {
            bail!("{} {}", &download_cfg.name, &download_cfg.version)
        };

        Ok(download_cfg)
    }
}
