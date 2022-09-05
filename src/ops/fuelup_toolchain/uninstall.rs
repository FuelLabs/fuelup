use std::str::FromStr;

use anyhow::Result;
use tracing::info;

use crate::{
    commands::toolchain::UninstallCommand,
    toolchain::{OfficialToolchainDescription, Toolchain},
};

pub fn uninstall(command: UninstallCommand) -> Result<()> {
    let UninstallCommand { name } = command;

    let mut toolchain = Toolchain::from(&name)?;

    if toolchain.is_official() {
        let description = OfficialToolchainDescription::from_str(&name)?;
        toolchain = Toolchain::from(&description.to_string())?;
    }

    if !toolchain.exists() {
        info!("toolchain '{}' does not exist", &toolchain.name);
        return Ok(());
    }

    toolchain.uninstall_self()?;
    info!("toolchain '{}' uninstalled", &toolchain.name);

    Ok(())
}
