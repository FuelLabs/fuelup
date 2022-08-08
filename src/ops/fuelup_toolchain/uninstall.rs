use anyhow::Result;
use tracing::info;

use crate::{commands::toolchain::UninstallCommand, toolchain::Toolchain};

pub fn uninstall(command: UninstallCommand) -> Result<()> {
    let UninstallCommand { name } = command;

    let toolchain = Toolchain::from(&name)?;

    if !toolchain.exists() {
        info!("toolchain '{}' does not exist", &name);
        return Ok(());
    }

    if toolchain.uninstall_self().is_ok() {
        info!("toolchain '{}' uninstalled", &name);
    };

    Ok(())
}
