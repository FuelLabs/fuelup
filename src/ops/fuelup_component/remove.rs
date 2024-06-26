use crate::{commands::component::RemoveCommand, toolchain::Toolchain};
use anyhow::{bail, Result};

pub fn remove(command: RemoveCommand) -> Result<()> {
    let RemoveCommand { component } = command;
    let toolchain = Toolchain::from_settings()?;

    if toolchain.is_distributed() {
        bail!(
            "Removing specific components is reserved for custom toolchains.
You are currently using '{}'.

You may create a custom toolchain using 'fuelup toolchain new <toolchain>'.",
            toolchain.name
        )
    };

    toolchain.remove_component(&component)?;
    Ok(())
}
