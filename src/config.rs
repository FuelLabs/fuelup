use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use std::io;

use crate::path::fuelup_dir;

pub struct Config {
    toolchains_dir: PathBuf,
}

impl Config {
    pub(crate) fn from_env() -> Result<Self> {
        let fuelup_dir = fuelup_dir();

        let toolchains_dir = fuelup_dir.join("toolchains");

        Ok(Self { toolchains_dir })
    }

    pub(crate) fn list_toolchains(&self) -> Result<Vec<String>> {
        if self.toolchains_dir.is_dir() {
            let toolchains: Vec<String> = fs::read_dir(&self.toolchains_dir)?
                .filter_map(io::Result::ok)
                .filter(|e| e.file_type().map(|f| f.is_dir()).unwrap_or(false))
                .map(|e| e.file_name().into_string().ok().unwrap_or_default())
                .collect();
            Ok(toolchains)
        } else {
            Ok(Vec::new())
        }
    }
}
