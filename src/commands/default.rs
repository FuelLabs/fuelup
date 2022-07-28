use anyhow::{bail, Result};
use clap::Parser;

use crate::{
    path::settings_file,
    settings::SettingsFile,
    toolchain::{toolchain, Toolchain},
};

#[derive(Debug, Parser)]
pub struct DefaultCommand {
    /// Set default toolchain.
    pub toolchain: Option<String>,
}

pub fn exec(command: DefaultCommand) -> Result<()> {
    let DefaultCommand { toolchain } = command;

    let settings = SettingsFile::new(settings_file());
    let current_default = settings.with(|s| Ok(s.default_toolchain.clone()));

    if toolchain.is_none() {
        if let Ok(Some(current_default_name)) = current_default {
            println!("{} (default)", current_default_name);
        }
        return Ok(());
    };

    let toolchain = toolchain.unwrap();
    let mut new_default = Toolchain::from(&toolchain)?;

    if [toolchain::LATEST].contains(&toolchain.as_str()) {
        new_default = Toolchain::new(&toolchain, None)?;
        println!("default toolchain set to '{}'", new_default.name);
    } else {
        if !new_default.path.exists() {
            bail!("Toolchain with name '{}' does not exist", &new_default.name)
        };
    };

    settings.with_mut(|s| {
        s.default_toolchain = Some(new_default.name.clone());
        Ok(())
    })?;

    Ok(())
}
