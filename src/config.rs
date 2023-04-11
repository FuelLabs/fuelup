use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use std::io;

use crate::fmt::format_toolchain_with_target;
use crate::path::toolchains_dir;
use crate::toolchain::RESERVED_TOOLCHAIN_NAMES;

pub struct Config {
    toolchains_dir: PathBuf,
}

impl Config {
    pub(crate) fn from_env() -> Result<Self> {
        Ok(Self {
            toolchains_dir: toolchains_dir(),
        })
    }

    pub(crate) fn list_toolchains(&self) -> Result<Vec<String>> {
        if self.toolchains_dir.is_dir() {
            let mut custom_toolchains: Vec<String> = vec![];
            let mut toolchains: Vec<String> = vec![];

            for dir_entry in fs::read_dir(&self.toolchains_dir)?
                .filter_map(io::Result::ok)
                .filter(|e| e.file_type().map(|f| f.is_dir()).unwrap_or(false))
            {
                let toolchain = dir_entry.file_name().to_string_lossy().to_string();
                if RESERVED_TOOLCHAIN_NAMES
                    .iter()
                    .any(|t| toolchain == format_toolchain_with_target(t))
                {
                    toolchains.push(toolchain)
                } else {
                    custom_toolchains.push(toolchain)
                }
            }

            toolchains.sort();
            custom_toolchains.sort();

            toolchains.extend(custom_toolchains);
            Ok(toolchains)
        } else {
            Ok(Vec::new())
        }
    }

    pub(crate) fn list_dist_toolchains(&self) -> Result<Vec<String>> {
        if self.toolchains_dir.is_dir() {
            let mut dist_toolchains: Vec<String> = Vec::new();
            let installed_toolchains: Vec<String> = fs::read_dir(&self.toolchains_dir)?
                .filter_map(io::Result::ok)
                .filter(|e| e.file_type().map(|f| f.is_dir()).unwrap_or(false))
                .map(|e| e.file_name().into_string().ok().unwrap_or_default())
                .collect();

            for name in RESERVED_TOOLCHAIN_NAMES {
                let dist_toolchain = format_toolchain_with_target(name);
                if installed_toolchains.contains(&dist_toolchain) {
                    dist_toolchains.push(name.to_string())
                }
            }

            Ok(dist_toolchains)
        } else {
            Ok(Vec::new())
        }
    }
}
