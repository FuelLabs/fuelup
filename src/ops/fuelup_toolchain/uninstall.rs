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

    toolchain.uninstall_self()?;
    info!("toolchain '{}' uninstalled", &name);

    Ok(())
}
