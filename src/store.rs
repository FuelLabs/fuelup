use std::path::{Path, PathBuf};

use anyhow::Result;
use semver::Version;

use crate::path::{ensure_dir_exists, store_dir};

pub struct Store {
    path: PathBuf,
}

impl Store {
    pub(crate) fn from_env() -> Self {
        Self { path: store_dir() }
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn into_path(&self) -> PathBuf {
        self.path.to_path_buf()
    }

    pub(crate) fn has_component(self, component_name: &str) -> Result<bool> {
        ensure_dir_exists(self.path())?;

        return Ok(self
            .into_path()
            .join(self.component_dirname(component_name, &Version::new(0, 0, 0)))
            .exists());
    }

    pub(crate) fn component_dirname(self, component_name: &str, version: &Version) -> String {
        format!("{component_name}-{version}")
    }
}
