use anyhow::{bail, Result};
use std::{fs, path::Path};
use tempfile::tempdir_in;
use tracing::{error, info};

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

    if let Err(e) = attempt_install_self(download_cfg, fuelup_new_dir.path()) {
        // Skip all other steps if downloading fails here.
        // We do not need to handle failure, since this downloads to a tempdir.
        bail!("Failed to install fuelup: {}", e);
    };

    let fuelup_backup = fuelup_backup_dir.path().join("fuelup-backup");
    if fuelup_bin.exists() {
        // Make a backup of fuelup, fuelup-backup.
        fs::copy(&fuelup_bin, &fuelup_backup).expect("Could not make a fuelup-backup");
        // On Linux, we have to unlink/remove the original bin first.
        fs::remove_file(&fuelup_bin).expect("Failed to remove fuelup");
    };

    info!(
        "Copying {} to {}",
        fuelup_new_dir.path().join("fuelup").display(),
        &fuelup_bin.display()
    );
    if let Err(e) = fs::copy(fuelup_new_dir.path().join("fuelup"), &fuelup_bin) {
        error!("Failed to replace the old fuelup: {}", e);

        // If we have failed to replace the old fuelup for whatever reason, we want the backup.
        // Should this last step fail, we will recommend to re-install fuelup using fuelup-init.
        if let Err(e) = fs::copy(&fuelup_backup, &fuelup_bin) {
            error!("Could not restore backup fuelup: {}", e);
            error!("You should re-install fuelup using fuelup-init:");
            error!("`curl --proto '=https' --tlsv1.2 -sSf https://fuellabs.github.io/fuelup/fuelup-init.sh | sh`");
        }
    };

    Ok(())
}
