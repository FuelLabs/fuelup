use anyhow::Result;
use clap::Parser;
use clap_complete::Shell;

use crate::ops::fuelup_completions;

/// Generate tab-completion scripts for your shell
#[derive(Debug, Parser)]
pub struct CompletionsCommand {
    /// Specify shell to enable tab-completion for
    ///
    /// [possible values: zsh, bash, fish, powershell, elvish]
    #[clap(short = 'S', long)]
    pub shell: Shell,
}

pub fn exec(command: CompletionsCommand) -> Result<()> {
    fuelup_completions::completions(command)?;

    Ok(())
}
