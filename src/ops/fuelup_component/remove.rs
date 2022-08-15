use anyhow::Result;

use crate::{commands::component::RemoveCommand, toolchain::Toolchain};

pub fn remove(command: RemoveCommand) -> Result<()> {
    let RemoveCommand { component } = command;

    let toolchain = Toolchain::from_settings()?;
    toolchain.remove_component(&component)?;
    Ok(())
}
