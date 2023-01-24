use std::path::{Path, PathBuf};

use anyhow::Result;
use semver::Version;

use crate::{
    download::{download_file_and_unpack, link_to_toolchain, unpack_bins, DownloadCfg},
    path::{ensure_dir_exists, store_dir},
};

fn component_dirname(component_name: &str, version: &Version) -> String {
    format!("{component_name}-{version}")
}

pub struct Store {
    path: PathBuf,
}

impl Store {
    pub(crate) fn from_env() -> Result<Self> {
        let path = store_dir();
        ensure_dir_exists(&path)?;
        Ok(Self { path })
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn has_component(&self, component_name: &str, version: &Version) -> bool {
        let dirname = component_dirname(component_name, version);
        println!("dirname: {}", self.path().join(dirname.clone()).display());
        println!("exists:{}", self.path().join(&dirname).exists());
        self.path().join(dirname).exists()
    }

    pub(crate) fn component_dir_path(&self, component_name: &str, version: &Version) -> PathBuf {
        self.path.join(component_dirname(component_name, version))
    }

    pub(crate) fn install_component(&self, cfg: &DownloadCfg) -> Result<()> {
        let component_dir = self.component_dir_path(&cfg.name, &cfg.version);

        ensure_dir_exists(&component_dir)?;
        download_file_and_unpack(cfg, &component_dir)?;
        // We ensure that component_dir exists above, so its parent must exist here.
        unpack_bins(&component_dir, &component_dir)?;

        Ok(())
    }

    // This function installs a component into a directory within '/.fuelup/store'.
    // The directory is named '<component_name>-<version>', eg. 'fuel-core-0.15.1'.
    pub(crate) fn install_toolchain_component(
        &self,
        toolchain_dir: PathBuf,
        cfg: &DownloadCfg,
    ) -> Result<()> {
        let component_dir = self.component_dir_path(&cfg.name, &cfg.version);

        ensure_dir_exists(&component_dir)?;
        download_file_and_unpack(cfg, &component_dir)?;
        // We ensure that component_dir exists above, so its parent must exist here.
        if let Ok(downloaded) = unpack_bins(&component_dir, &component_dir) {
            ensure_dir_exists(&toolchain_dir)?;
            ensure_dir_exists(&toolchain_dir.join("bin"))?;
            link_to_toolchain(toolchain_dir, downloaded)?;
        }

        Ok(())
    }
}
