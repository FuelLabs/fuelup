use crate::{constants::FUEL_TOOLCHAIN_TOML_FILE, fmt::println_warn};
use anyhow::{bail, Result};
use component::Components;
use dirs;
use std::env;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub const FUELUP_DIR: &str = ".fuelup";
pub const FUELUP_HOME: &str = "FUELUP_HOME";

pub fn fuelup_dir() -> PathBuf {
    dirs::home_dir().unwrap().join(FUELUP_DIR)
}

pub fn fuelup_bin_dir() -> PathBuf {
    fuelup_dir().join("bin")
}

pub fn fuelup_bin() -> PathBuf {
    fuelup_bin_dir().join("fuelup")
}

pub fn fuelup_log_dir() -> PathBuf {
    fuelup_dir().join("log")
}

pub fn settings_file() -> PathBuf {
    fuelup_dir().join("settings.toml")
}

pub fn hashes_dir() -> PathBuf {
    fuelup_dir().join("hashes")
}

pub fn toolchains_dir() -> PathBuf {
    fuelup_dir().join("toolchains")
}

pub fn store_dir() -> PathBuf {
    fuelup_dir().join("store")
}

pub fn fuelup_tmp_dir() -> PathBuf {
    fuelup_dir().join("tmp")
}

pub fn toolchain_dir(toolchain: &str) -> PathBuf {
    toolchains_dir().join(toolchain)
}

pub fn toolchain_bin_dir(toolchain: &str) -> PathBuf {
    toolchain_dir(toolchain).join("bin")
}

pub fn ensure_dir_exists(path: &Path) -> Result<()> {
    if !path.is_dir() {
        fs::create_dir_all(path)
            .or_else(|e| bail!("Failed to create directory {}: {}", path.display(), e))?
    }
    Ok(())
}

pub fn warn_existing_fuel_executables() -> Result<()> {
    let components = Components::collect_publishables()?.into_iter();

    fn search_directories() -> Vec<PathBuf> {
        if let Some(val) = env::var_os("PATH") {
            return env::split_paths(&val).collect();
        }
        vec![]
    }

    for c in components {
        for e in c.executables {
            if let Some(path) = search_directories()
                .into_iter()
                .map(|d| d.join(e.clone()))
                .find(|f| is_executable(f))
            {
                let path = path.to_str().unwrap_or_default();
                let mut message = String::new();
                let fuelup_bin_dir = fuelup_bin_dir();
                if !path.contains(fuelup_bin_dir.to_str().unwrap()) {
                    let maybe_fuelup_executable = fuelup_bin_dir.join(&e);

                    message.push_str(&format!("warning: '{e}' found in PATH at {path}."));

                    if is_executable(&maybe_fuelup_executable) {
                        message.push_str(&format!(
                            " This will take precedence over '{}', already installed at {}. Consider uninstalling {}, or re-arranging your PATH to give fuelup priority.",
                            c.name,
                            &maybe_fuelup_executable.display(),
                            path
                        ));
                    } else {
                        message.push_str(&format!(
                            " This will take precedence over '{}' to be installed at {}.",
                            c.name,
                            &maybe_fuelup_executable.display()
                        ));
                    }
                }

                if let Ok(cargo_home) = std::env::var("CARGO_HOME") {
                    if path.contains(&cargo_home) {
                        message.push_str(&format!(
                            " You may want to execute 'cargo uninstall {}'.",
                            c.name
                        ));
                    }
                }

                if !message.is_empty() {
                    println_warn(message);
                }
            }
        }
    }

    Ok(())
}

fn find_parent_dir_with_file(starter_path: &Path, file_name: &str) -> Option<PathBuf> {
    let mut path = std::fs::canonicalize(starter_path).ok()?;
    let empty_path = PathBuf::from("/");
    while path != empty_path {
        path.push(file_name);
        if path.exists() {
            path.pop();
            return Some(path);
        } else {
            path.pop();
            path.pop();
        }
    }
    None
}

pub fn get_fuel_toolchain_toml() -> Option<PathBuf> {
    let parent_dir =
        find_parent_dir_with_file(&std::env::current_dir().unwrap(), FUEL_TOOLCHAIN_TOML_FILE);
    parent_dir.map(|p| p.join(FUEL_TOOLCHAIN_TOML_FILE))
}

#[cfg(unix)]
pub fn is_executable(path: &Path) -> bool {
    use std::os::unix::prelude::*;
    std::fs::metadata(path)
        .map(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(windows)]
pub fn is_executable(path: &Path) -> bool {
    path.is_file()
}
