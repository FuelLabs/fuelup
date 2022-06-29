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
        // Make a backup of fuelup.
        fs::copy(&fuelup_bin, &fuelup_backup).expect("Could not make a fuelup backup");
        // We cannot copy new fuelup over the old; we must unlink it first.
        fs::remove_file(&fuelup_bin).expect("Could not remove fuelup");
    };

    if let Err(e) = attempt_install_self(download_cfg, &fuelup_bin_dir()) {
        error!("Failed to install and replace fuelup. {}", e);

        // We want to restore the backup fuelup in case download goes wrong.
        if fuelup_backup.exists() {
            fs::copy(&fuelup_backup, &fuelup_bin).expect("Could not restore fuelup-backup");
        }
    };

    if fuelup_backup.exists() {
        fs::remove_file(&fuelup_backup).expect("Could not remove fuelup backup");
    }

    Ok(())
}
