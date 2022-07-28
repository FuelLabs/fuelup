use anyhow::Result;

use crate::commands::component::RemoveCommand;

pub fn remove(command: RemoveCommand) -> Result<()> {
    let RemoveCommand { component } = command;
    println!("Remove component {}", component);
    Ok(())
}
