use anyhow::{bail, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};

use dirs;

pub const FUELUP_DIR: &str = ".fuelup";

pub fn fuelup_dir() -> PathBuf {
    dirs::home_dir().unwrap().join(FUELUP_DIR)
}

pub fn fuelup_bin_dir() -> PathBuf {
    fuelup_dir().join("bin")
}

pub fn fuelup_bin() -> PathBuf {
    fuelup_bin_dir().join("fuelup")
}

pub fn settings_file() -> PathBuf {
    fuelup_dir().join("settings.toml")
}

pub fn toolchain_dir() -> PathBuf {
    fuelup_dir().join("toolchains")
}

pub fn toolchain_bin_dir(toolchain: &str) -> PathBuf {
    toolchain_dir().join(toolchain).join("bin")
}

pub fn ensure_dir_exists(path: &Path) -> Result<()> {
    if !path.is_dir() {
        fs::create_dir_all(path).or_else(|e| bail!("Failed to create {}: {}", path.display(), e))?
    }
    Ok(())
}
