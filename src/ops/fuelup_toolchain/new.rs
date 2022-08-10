use crate::path::{ensure_dir_exists, settings_file, toolchain_bin_dir};
use crate::settings::SettingsFile;
use crate::{commands::toolchain::NewCommand, path::toolchain_dir};
use anyhow::bail;
use anyhow::Result;
use std::fs;
use std::io;
use tracing::info;

pub fn new(command: NewCommand) -> Result<()> {
    let NewCommand { name } = command;

    let toolchain_dir = toolchain_dir();

    let toolchain_exists = fs::read_dir(&toolchain_dir)?
        .filter_map(io::Result::ok)
        .filter(|e| e.file_type().map(|f| f.is_dir()).unwrap_or(false))
        .map(|e| e.file_name().into_string().ok().unwrap_or_default())
        .any(|x| x == name);

    if toolchain_exists {
        bail!("Toolchain with name '{}' already exists", &name)
    }

    let toolchain_bin_dir = toolchain_bin_dir(&name);

    let settings_file = settings_file();
    if !settings_file.exists() {
        let settings = SettingsFile::new(settings_file);
        settings.with_mut(|s| {
            s.default_toolchain = Some(name.clone());
            Ok(())
        })?;
    }

    if ensure_dir_exists(&toolchain_dir.join(toolchain_bin_dir)).is_ok() {
        info!("New toolchain initialized: {}", &name);
    };

    Ok(())
}
