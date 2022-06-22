use std::path::PathBuf;

use dirs;

pub const FUELUP_DIR: &str = ".fuelup";

pub fn fuelup_dir() -> PathBuf {
    dirs::home_dir().unwrap().join(FUELUP_DIR)
}

pub fn fuelup_bin_dir() -> PathBuf {
    fuelup_dir().join("bin")
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
