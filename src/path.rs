use std::path::PathBuf;

use dirs;

pub const FUELUP_DIR: &str = ".fuelup";

pub fn fuelup_bin_dir() -> PathBuf {
    dirs::home_dir().unwrap().join(FUELUP_DIR).join("bin")
}

pub fn toolchain_bin_dir(toolchain: &str) -> PathBuf {
    dirs::home_dir()
        .unwrap()
        .join(FUELUP_DIR)
        .join(toolchain)
        .join("bin")
}
