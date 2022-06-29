use anyhow::Result;
use std::{fs, path::Path};
use tempfile::tempdir_in;
use tracing::error;

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
    let fuelup_bin_dir = fuelup_bin_dir();

    let fuelup_new_dir = tempdir_in(&fuelup_bin_dir)?;
    let fuelup_backup_dir = tempdir_in(&fuelup_bin_dir)?;

    if let Err(e) = attempt_install_self(download_cfg, &fuelup_new_dir.path()) {
        error!("Failed to install and replace fuelup. {}", e);
    };

    let fuelup_backup = fuelup_backup_dir.path().join("fuelup-backup");
    if fuelup_bin.exists() {
        // Make a backup of fuelup, fuelup-backup.
        fs::copy(&fuelup_bin, &fuelup_backup).expect("Could not make a fuelup-backup");
        fs::remove_file(&fuelup_bin).expect("Failed to remove fuelup");
    };

    // Copy the new fuelup into the bin folder.
    if let Err(e) = fs::copy(fuelup_new_dir.path().join("fuelup"), &fuelup_bin) {
        // If we have failed to replace the old fuelup for whatever reason, we want the backup.
        error!("Failed to replace the old fuelup: {}", e);
        if let Err(e) = fs::copy(&fuelup_backup, &fuelup_bin) {
            error!("Could not restore backup fuelup: {}. You should re-install fuelup using the script: `curl --proto '=https' --tlsv1.2 -sSf https://fuellabs.github.io/fuelup/fuelup-init.sh | sh`", e);
        }
    };

    // Finally remove backup and the folder.
    let _ = fs::remove_file(&fuelup_backup);

    Ok(())
}
