use anyhow::Result;
use clap::Parser;

use crate::ops::fuelup_component::{add::add, remove::remove};

#[derive(Debug, Parser)]
pub enum ComponentCommand {
    /// Add a component to component to the currently active Fuel toolchain
    Add(AddCommand),
    /// Remove a component from the currently active Fuel toolchain
    Remove(RemoveCommand),
}

#[derive(Debug, Parser)]
pub struct AddCommand {
    /// Component name [possible values: forc, forc@<version>, fuel-core, fuel-core@<version>]
    pub maybe_versioned_component: String,
}

#[derive(Debug, Parser)]
pub struct RemoveCommand {
    /// Component name [possible values: forc, fuel-core]
    pub component: String,
}

pub fn exec(command: ComponentCommand) -> Result<()> {
    match command {
        ComponentCommand::Add(command) => add(command)?,
        ComponentCommand::Remove(command) => remove(command)?,
    };

    Ok(())
}
