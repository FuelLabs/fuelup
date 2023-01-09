use std::path::{Path, PathBuf};

use anyhow::Result;
use semver::Version;

use crate::{
    download::{download_file_and_unpack, unpack_bins, DownloadCfg},
    path::{ensure_dir_exists, store_dir},
    target_triple::TargetTriple,
    toolchain_override::ToolchainOverride,
};

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

    pub(crate) fn has_component(
        &self,
        component_name: &str,
        version: Option<&Version>,
    ) -> Result<bool> {
        if version.is_none() {
            return Ok(false);
        }
        ensure_dir_exists(self.path())?;

        let dirname = self.component_dirname(component_name, version.unwrap());
        Ok(self.path().join(dirname).exists())
    }

    pub(crate) fn component_dirname(&self, component_name: &str, version: &Version) -> String {
        format!("{component_name}-{version}")
    }

    pub(crate) fn component_dir_path(&self, component_name: &str, version: &Version) -> PathBuf {
        self.path
            .join(self.component_dirname(component_name, version))
    }

    // This function installs a component into a directory within '/.fuelup/store'.
    // The directory is named '<component_name>-<version>', eg. 'fuel-core-0.15.1'.
    pub(crate) fn install_component(
        &self,
        component_name: &str,
        toolchain_override: &ToolchainOverride,
    ) -> Result<()> {
        if let Some(components) = toolchain_override.cfg.components.as_ref() {
            let version = components.get(component_name).map(|v| v.clone());

            let download_cfg = DownloadCfg::new(
                component_name,
                TargetTriple::from_component(component_name).unwrap(),
                version.clone(),
            )?;

            let component_dir = self.component_dir_path(&component_name, &version.unwrap());
            ensure_dir_exists(&component_dir)?;
            download_file_and_unpack(&download_cfg, &component_dir)?;
            unpack_bins(&component_dir, &component_dir.parent().unwrap())?;
        }

        Ok(())
    }
}
