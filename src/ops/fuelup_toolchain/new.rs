use crate::commands::toolchain::NewCommand;
use crate::path::{ensure_dir_exists, settings_file, toolchain_bin_dir, toolchains_dir};
use crate::settings::SettingsFile;
use crate::toolchain::Toolchain;
use anyhow::bail;
use anyhow::Result;
use tracing::info;

pub fn new(command: NewCommand) -> Result<()> {
    let NewCommand { name } = command;

    let toolchains_dir = toolchains_dir();

    let toolchain_exists = Toolchain::all()?.into_iter().any(|x| x == name);

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
Default toolchain set to '{name}'"
    );

    Ok(())
}
