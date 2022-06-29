use std::{env::temp_dir, fs, path::Path};
use tracing::error;

use anyhow::Result;

use crate::{
    component,
    download::{download_file_and_unpack, unpack_bins, DownloadCfg},
    path::{fuelup_bin, fuelup_bin_dir},
};

pub fn attempt_install_self(download_cfg: DownloadCfg, dst: &Path) -> Result<()> {
    download_file_and_unpack(&download_cfg, dst)?;
    unpack_bins(dst, dst)?;

    Ok(())
}

pub fn self_update() -> Result<()> {
    let download_cfg = DownloadCfg::new(component::FUELUP, None)?;
    let fuelup_bin = fuelup_bin();
    let fuelup_backup = temp_dir().join("fuelup-backup");

    if fuelup_bin.exists() {
        // Make a backup of fuelup, in case downloading fails. If download fails below, we use the backup.
        fs::copy(&fuelup_bin, &fuelup_backup).expect("Could not make a fuelup backup");
        // We cannot copy new fuelup over the old; we must unlink it first.
        fs::remove_file(&fuelup_bin).expect("Could not remove fuelup");
    };

    if let Err(e) = attempt_install_self(download_cfg, &fuelup_bin_dir()) {
        error!("Failed to install and replace fuelup - {}.", e);
        fs::copy(fuelup_backup, fuelup_bin)?;
    };

    Ok(())
}
