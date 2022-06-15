use anyhow::Result;

use crate::{
    component,
    download::{download_file_and_unpack, unpack_bins, DownloadCfg},
    path::fuelup_bin_dir,
};

pub fn self_update() -> Result<()> {
    let download_cfg = DownloadCfg::new(component::FUELUP, None)?;
    let fuelup_bin_dir = fuelup_bin_dir();

    download_file_and_unpack(&download_cfg, &fuelup_bin_dir)?;
    unpack_bins(&fuelup_bin_dir, &fuelup_bin_dir)?;

    Ok(())
}
