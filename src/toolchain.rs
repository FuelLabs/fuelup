use anyhow::{bail, Result};
use std::path::PathBuf;

use crate::download::{component, DownloadCfg};

pub mod toolchain {
    pub const LATEST: &str = "latest";
}

pub struct Toolchain {
    pub name: String,
    pub path: PathBuf,
}

impl Toolchain {
    pub fn from(name: &str) -> Result<Self> {
        let path = match name {
            toolchain::LATEST => PathBuf::from(""),
            _ => bail!("Unknown toolchain: {}", name),
        };
        Ok(Self {
            name: name.to_string(),
            path,
        })
    }
}
