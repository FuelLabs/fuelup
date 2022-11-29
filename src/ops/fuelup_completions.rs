use crate::commands::completions::CompletionsCommand;
use anyhow::Result;
use clap::CommandFactory;
use clap_complete::generate;

pub fn completions(command: CompletionsCommand) -> Result<()> {
    let mut cmd = super::super::fuelup_cli::Cli::command();
    let bin_name = cmd.get_name().to_string();
    generate(command.shell, &mut cmd, bin_name, &mut std::io::stdout());
    Ok(())
}
