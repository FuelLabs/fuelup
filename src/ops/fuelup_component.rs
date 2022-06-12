use std::fs;

use anyhow::{bail, Result};
use tracing::info;

use crate::{
    download::{download_file_and_unpack, unpack_extracted_bins, DownloadCfg},
    path::fuelup_bin_dir,
};

pub fn install_one(download_cfg: DownloadCfg) -> Result<DownloadCfg> {
    let fuelup_bin_dir = fuelup_bin_dir();
    if !fuelup_bin_dir.is_dir() {
        fs::create_dir_all(&fuelup_bin_dir).expect("Unable to create fuelup directory");
    }

    info!("Fetching {} {}", &download_cfg.name, &download_cfg.version);

    if download_file_and_unpack(&download_cfg).is_err() {
        bail!("{} {}", &download_cfg.name, &download_cfg.version)
    };

    if unpack_extracted_bins(&fuelup_bin_dir).is_err() {
        bail!("{} {}", &download_cfg.name, &download_cfg.version)
    };

    Ok(download_cfg)
}
