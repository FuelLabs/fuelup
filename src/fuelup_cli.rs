use anyhow::Result;
use clap::Parser;

use crate::commands::show::ShowCommand;
use crate::commands::{check, completions, component, default, fuelup, show, toolchain, update};

use crate::commands::check::CheckCommand;
use crate::commands::completions::CompletionsCommand;
use crate::commands::component::ComponentCommand;
use crate::commands::default::DefaultCommand;
use crate::commands::fuelup::FuelupCommand;
use crate::commands::toolchain::ToolchainCommand;
use crate::commands::update::UpdateCommand;

#[derive(Debug, Parser)]
#[clap(name = "fuelup", about = "Fuel Toolchain Manager", version)]
pub struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Parser)]
enum Commands {
    /// Check for updates to Fuel toolchains and fuelup
    Check(CheckCommand),
    /// Generate shell competions
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
}

pub fn fuelup_cli() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Check(command) => check::exec(command),
        Commands::Completions(command) => completions::exec(command),
        Commands::Component(command) => component::exec(command),
        Commands::Default_(command) => default::exec(command),
        Commands::Fuelup(command) => match command {
            FuelupCommand::Update => fuelup::exec(),
        },
        Commands::Show(_command) => show::exec(),
        Commands::Toolchain(command) => toolchain::exec(command),
        Commands::Update(_command) => update::exec(),
    }
}
