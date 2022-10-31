use anyhow::{bail, Result};
use component;
use std::{fs, path::Path};
use tempfile;
use tracing::{error, info};

use crate::{
    download::{download_file_and_unpack, unpack_bins, DownloadCfg},
    path::fuelup_bin,
    target_triple::TargetTriple,
};

pub fn attempt_install_self(download_cfg: DownloadCfg, dst: &Path) -> Result<()> {
    download_file_and_unpack(&download_cfg, dst)?;
    unpack_bins(dst, dst)?;

    Ok(())
}

pub fn self_update() -> Result<()> {
    let download_cfg = DownloadCfg::new(
        component::FUELUP,
        TargetTriple::from_component(component::FUELUP)?,
        None,
    )?;

    let fuelup_bin = fuelup_bin();

    let fuelup_new_dir = tempfile::tempdir()?;
    let fuelup_new_dir_path = fuelup_new_dir.path();
    let fuelup_backup = fuelup_new_dir_path.join("fuelup-backup");
    let fuelup_new = fuelup_new_dir_path.join(component::FUELUP);

    if let Err(e) = attempt_install_self(download_cfg, fuelup_new_dir_path) {
        // Skip all other steps if downloading fails here.
        // We do not need to handle failure, since this downloads to a tempdir.
        bail!("Failed to install fuelup: {}", e);
    };

    if fuelup_bin.exists() {
        if fuelup_backup.exists() {
            fs::remove_file(&fuelup_backup)?;
        };

        // Make a backup of fuelup within /tmp, in case download fails.
        if let Err(e) = fs::copy(&fuelup_bin, &fuelup_backup) {
            bail!(
                "Failed moving {} to {}: {}",
                &fuelup_bin.display(),
                &fuelup_backup.display(),
                e
            );
        }

        // Unlink the original 'fuelup', since we cannot write over a running executable.
        fs::remove_file(&fuelup_bin)?;
    }

    info!(
        "Moving {} to {}",
        fuelup_new.display(),
        &fuelup_bin.display()
    );
    if let Err(e) = fs::copy(&fuelup_new, &fuelup_bin) {
        error!(
            "Failed to replace old fuelup with new fuelup: {}. Attempting to restore backup fuelup.",
        e);
        // If we have failed to replace the old fuelup for whatever reason, we want the backup.
        // Although unlikely, should this last step fail, we will recommend to re-install fuelup using fuelup-init.
        if let Err(e) = fs::copy(&fuelup_backup, &fuelup_bin) {
            bail!(
                "Could not restore backup fuelup. {}

You should re-install fuelup using fuelup-init:
`curl --proto '=https' --tlsv1.2 -sSf https://fuellabs.github.io/fuelup/fuelup-init.sh | sh`",
                e
            );
        } else {
            let _ = fs::remove_file(&fuelup_backup);
        }

        bail!(
            "Old fuelup restored because something went wrong replacing old fuelup with new fuelup.",
        );
    } else {
        let _ = fs::remove_file(&fuelup_new);
    };

    // Remove backup at the end.
    let _ = fs::remove_file(&fuelup_backup);

    Ok(())
}
