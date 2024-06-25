use crate::ops::fuelup_component::{add::add, list::list, remove::remove};
use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
pub enum ComponentCommand {
    /// Add a component to the currently active custom toolchain.
    Add(AddCommand),
    /// Remove a component from the currently active custom toolchain
    Remove(RemoveCommand),
    /// List installed and installable components
    List,
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
        ComponentCommand::List => list()?,
    };
    Ok(())
}
