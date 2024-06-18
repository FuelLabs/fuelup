use anyhow::Result;
use clap::Parser;

use crate::commands::show::ShowCommand;
use crate::commands::{completions, component, default, fuelup, show, toolchain, update, upgrade};

use crate::commands::completions::CompletionsCommand;
use crate::commands::component::ComponentCommand;
use crate::commands::default::DefaultCommand;
use crate::commands::fuelup::FuelupCommand;
use crate::commands::toolchain::ToolchainCommand;
use crate::commands::update::UpdateCommand;
use crate::commands::upgrade::UpgradeCommand;
use crate::ops::fuelup_check;

#[derive(Debug, Parser)]
#[clap(name = "fuelup", about = "Fuel Toolchain Manager", version)]
pub struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Parser)]
enum Commands {
    /// Check for updates to Fuel toolchains and fuelup
    Check,
    /// Generate shell completions
    Completions(CompletionsCommand),
    /// Add or remove components from the currently active toolchain
    #[clap(subcommand)]
    Component(ComponentCommand),
    /// Set default toolchain
    Default_(DefaultCommand),
    /// Manage your fuelup installation.
    #[clap(name = "self", subcommand)]
    Fuelup(FuelupCommand),
    /// Install new toolchains or modify/query installed toolchains
    #[clap(subcommand)]
    Toolchain(ToolchainCommand),
    /// Show the active and installed toolchains, as well as the host and fuelup home
    Show(ShowCommand),
    /// Updates the distributable toolchains, if already installed
    Update(UpdateCommand),
    /// Updates fuelup itself, switches to the `latest` channel and updates components in all channels.
    Upgrade(UpgradeCommand),
}

pub fn fuelup_cli() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Check => fuelup_check::check(),
        Commands::Completions(command) => completions::exec(command),
        Commands::Component(command) => component::exec(command),
        Commands::Default_(command) => default::exec(command),
        Commands::Fuelup(command) => match command {
            FuelupCommand::Update(update) => fuelup::update_exec(update.force),
            FuelupCommand::Uninstall(remove) => fuelup::remove_exec(remove.force),
        },
        Commands::Show(_command) => show::exec(),
        Commands::Toolchain(command) => toolchain::exec(command),
        Commands::Update(_command) => update::exec(),
        Commands::Upgrade(command) => upgrade::exec(command.force),
    }
}
