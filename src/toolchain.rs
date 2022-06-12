use anyhow::{bail, Result};
use std::{fs, ops::Deref, path::PathBuf, str::FromStr};
use tracing::info;

use crate::{
    download::{component, download_file_and_unpack, unpack_extracted_bins, DownloadCfg},
    path::{fuelup_bin_dir, FUELUP_DIR},
};

pub mod toolchain {
    pub const LATEST: &str = "latest";
}

pub struct Toolchain {
    pub name: String,
    pub path: PathBuf,
    pub target: TargetTriple,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TargetTriple(String);

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
        let path = match name {
            toolchain::LATEST => dirs::home_dir()
                .unwrap()
                .join(FUELUP_DIR)
                .join("toolchains")
                .join("latest-x86_64-apple-darwin")
                .join("bin"),
            _ => bail!("Unknown toolchain: {}", name),
        };
        Ok(Self {
            name: name.to_string(),
            path,
            target,
        })
    }

    pub fn add_component(&self, download_cfg: DownloadCfg) -> Result<DownloadCfg> {
        if !self.path.is_dir() {
            fs::create_dir_all(&self.path).expect("Unable to create fuelup directory");
        }

        info!("Fetching {} {}", &download_cfg.name, &download_cfg.version);

        if download_file_and_unpack(&download_cfg, &self.path).is_err() {
            bail!("{} {}", &download_cfg.name, &download_cfg.version)
        };

        if unpack_extracted_bins(&self.path).is_err() {
            bail!("{} {}", &download_cfg.name, &download_cfg.version)
        };

        Ok(download_cfg)
    }
}
