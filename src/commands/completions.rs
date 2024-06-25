use crate::ops::fuelup_completions;
use anyhow::Result;
use clap::Parser;
use clap_complete::Shell;

/// Generate tab-completion scripts for your shell
#[derive(Debug, Parser)]
pub struct CompletionsCommand {
    /// Specify shell to enable tab-completion for
    #[clap(short = 'S', long)]
    pub shell: Shell,
}

pub fn exec(command: CompletionsCommand) -> Result<()> {
    fuelup_completions::completions(command)?;
    Ok(())
}
