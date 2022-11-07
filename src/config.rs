use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
use std::io::{self, ErrorKind};

use crate::file::write_file;
use crate::fmt::format_toolchain_with_target;
use crate::path::{ensure_dir_exists, hashes_dir, toolchains_dir};
use crate::toolchain::{OfficialToolchainDescription, RESERVED_TOOLCHAIN_NAMES};

pub struct Config {
    toolchains_dir: PathBuf,
    hashes_dir: PathBuf,
}

impl Config {
    pub(crate) fn from_env() -> Result<Self> {
        Ok(Self {
            toolchains_dir: toolchains_dir(),
            hashes_dir: hashes_dir(),
        })
    }

    pub(crate) fn hashes_dir(&self) -> &Path {
        self.hashes_dir.as_path()
    }

    pub(crate) fn hash_matches(
        &self,
        description: &OfficialToolchainDescription,
        hash: &str,
    ) -> Result<bool> {
        let hash_path = self.hashes_dir.join(description.to_string());

        match fs::read_to_string(&hash_path) {
            Ok(h) => Ok(h == hash),
            Err(e) => match e.kind() {
                ErrorKind::NotFound => Ok(false),
                _ => bail!("Failed to read hash from hash file"),
            },
        }
    }

    pub(crate) fn hash_exists(&self, description: &OfficialToolchainDescription) -> bool {
        self.hashes_dir.join(description.to_string()).is_file()
    }

    pub(crate) fn save_hash(&self, toolchain: &str, hash: &str) -> Result<()> {
        ensure_dir_exists(&self.hashes_dir)?;
        write_file(&self.hashes_dir.join(toolchain), hash)?;
        Ok(())
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
                    .any(|t| toolchain == format_toolchain_with_target(&t))
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

    pub(crate) fn list_official_toolchains(&self) -> Result<Vec<String>> {
        if self.toolchains_dir.is_dir() {
            let toolchains: Vec<String> = fs::read_dir(&self.toolchains_dir)?
                .filter_map(io::Result::ok)
                .filter(|e| {
                    e.file_type().map(|f| f.is_dir()).unwrap_or(false)
                        && RESERVED_TOOLCHAIN_NAMES.iter().any(|t| {
                            e.file_name().to_string_lossy() == format_toolchain_with_target(&t)
                        })
                })
                .map(|e| e.file_name().into_string().ok().unwrap_or_default())
                .collect();
            Ok(toolchains)
        } else {
            Ok(Vec::new())
        }
    }
}
