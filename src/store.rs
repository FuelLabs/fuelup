use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::Result;
use component::Component;
use semver::Version;
use tracing::{info, warn};

use crate::{
    download::{download_file_and_unpack, fetch_fuels_version, unpack_bins, DownloadCfg},
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
        self.path().join(dirname).exists()
    }

    pub(crate) fn component_dir_path(&self, component_name: &str, version: &Version) -> PathBuf {
        self.path.join(component_dirname(component_name, version))
    }

    // This function installs a component into a directory within '/.fuelup/store'.
    // The directory is named '<component_name>-<version>', eg. 'fuel-core-0.15.1'.
    pub(crate) fn install_component(&self, cfg: &DownloadCfg) -> Result<Vec<PathBuf>> {
        let component_dir = self.component_dir_path(&cfg.name, &cfg.version);

        // Cache fuels_version for this component, if show_fuels_version exists and is true.
        // We don't want this failure to block installation, so errors are ignored here.
        Component::from_name(&cfg.name).ok().map(|c| {
            if let Some(true) = c.show_fuels_version {
                if let Err(e) = self.cache_fuels_version(cfg) {
                    warn!(
                        "Failed to cache fuels version for component '{}': {}",
                        cfg.name, e
                    );
                };
            }
        });

        ensure_dir_exists(&component_dir)?;
        download_file_and_unpack(cfg, &component_dir)?;
        // We ensure that component_dir exists above, so its parent must exist here.
        unpack_bins(&component_dir, &component_dir)
    }

    pub(crate) fn cache_fuels_version(&self, cfg: &DownloadCfg) -> Result<()> {
        let dirname = component_dirname(&cfg.name, &cfg.version);

        if let Ok(fuels_version) = fetch_fuels_version(cfg) {
            info!("caching fuels version");
            ensure_dir_exists(&self.path().join(&dirname))?;
            let fuels_version_path = self.path().join(dirname).join("fuels_version");
            let mut fuels_version_file = std::fs::File::create(fuels_version_path)?;

            fuels_version_file.write(&format!("{fuels_version}").into_bytes())?;
        };

        Ok(())
    }

    pub(crate) fn get_cached_fuels_version(
        &self,
        name: &str,
        version: &Version,
    ) -> std::io::Result<String> {
        let dirname = component_dirname(name, version);

        fs::read_to_string(self.path().join(dirname).join("fuels_version"))
    }
}
