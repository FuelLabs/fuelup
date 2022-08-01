use crate::toolchain::RESERVED_TOOLCHAIN_NAMES;
use anyhow::{bail, Result};
use clap::Parser;

use crate::{path::settings_file, settings::SettingsFile, toolchain::Toolchain};

#[derive(Debug, Parser)]
pub struct DefaultCommand {
    /// Set default toolchain.
    pub toolchain: Option<String>,
}

pub fn exec(command: DefaultCommand) -> Result<()> {
    let DefaultCommand { toolchain } = command;

    let current_toolchain = Toolchain::from_settings()?;

    if toolchain.is_none() {
        println!("{} (default)", current_toolchain.name);
        return Ok(());
    };

    let toolchain = toolchain.unwrap();
    let mut new_default = Toolchain::from(&toolchain)?;

    if RESERVED_TOOLCHAIN_NAMES.contains(&toolchain.as_str()) {
        new_default = Toolchain::new(&toolchain, None)?;
    } else if !new_default.path.exists() {
        bail!("Toolchain with name '{}' does not exist", &new_default.name)
    };

    let settings = SettingsFile::new(settings_file());
    settings.with_mut(|s| {
        s.default_toolchain = Some(new_default.name.clone());
        Ok(())
    })?;
    println!("default toolchain set to '{}'", new_default.name);

    Ok(())
}
