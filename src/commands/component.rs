use anyhow::Result;
use clap::Parser;

use crate::ops::fuelup_component::{add::add, list::list, remove::remove};

#[derive(Debug, Parser)]
pub enum ComponentCommand {
    /// Add a component to the currently active custom toolchain.
    Add(AddCommand),
    /// Remove a component from the currently active custom toolchain
    Remove(RemoveCommand),
    /// List installed and installable components
    List(ListCommand),
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

#[derive(Debug, Parser)]
pub struct ListCommand;

pub fn exec(command: ComponentCommand) -> Result<()> {
    match command {
        ComponentCommand::Add(command) => add(command)?,
        ComponentCommand::Remove(command) => remove(command)?,
        ComponentCommand::List(command) => list(command)?,
    };

    Ok(())
}
