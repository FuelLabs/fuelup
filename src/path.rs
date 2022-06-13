use std::path::PathBuf;

use dirs;

pub const FUELUP_DIR: &str = ".fuelup";

pub fn fuelup_dir() -> PathBuf {
    dirs::home_dir().unwrap().join(FUELUP_DIR)
}

pub fn fuelup_bin_dir() -> PathBuf {
    dirs::home_dir().unwrap().join(FUELUP_DIR).join("bin")
}

pub fn settings_file() -> PathBuf {
    dirs::home_dir()
        .unwrap()
        .join(FUELUP_DIR)
        .join("settings.toml")
}

pub fn toolchain_bin_dir(toolchain: &str) -> PathBuf {
    dirs::home_dir()
        .unwrap()
        .join(FUELUP_DIR)
        .join("toolchains")
        .join(toolchain)
        .join("bin")
}
