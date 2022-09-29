use crate::commands::toolchain::NewCommand;
use crate::path::{ensure_dir_exists, settings_file, toolchain_bin_dir, toolchains_dir};
use crate::settings::SettingsFile;
use anyhow::bail;
use anyhow::Result;
use std::fs;
use std::io;
use tracing::info;

pub fn new(command: NewCommand) -> Result<()> {
    let NewCommand { name } = command;

    let toolchains_dir = toolchains_dir();

    let toolchain_exists = toolchains_dir.is_dir()
        && fs::read_dir(&toolchains_dir)?
            .filter_map(io::Result::ok)
            .filter(|e| e.path().is_dir())
            .map(|e| e.file_name().into_string().ok().unwrap_or_default())
            .any(|x| x == name);

    if toolchain_exists {
        bail!("Toolchain with name '{}' already exists", &name)
    }

    let toolchain_bin_dir = toolchain_bin_dir(&name);

    let settings_file = settings_file();

    let settings = SettingsFile::new(settings_file);
    settings.with_mut(|s| {
        s.default_toolchain = Some(name.clone());
        Ok(())
    })?;

    ensure_dir_exists(&toolchains_dir.join(toolchain_bin_dir))?;
    info!(
        "New toolchain initialized: {name}
default toolchain set to '{name}'"
    );

    Ok(())
}
