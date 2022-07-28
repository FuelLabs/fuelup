use anyhow::Result;

use crate::commands::component::AddCommand;

pub fn add(command: AddCommand) -> Result<()> {
    let AddCommand { component } = command;
    println!("Add component {}", component);
    Ok(())
}
