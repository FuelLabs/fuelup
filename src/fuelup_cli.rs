use anyhow::Result;
use clap::Parser;

use crate::commands::show::ShowCommand;
use crate::commands::{check, component, default, fuelup, show, toolchain};

use crate::commands::check::CheckCommand;
use crate::commands::component::ComponentCommand;
use crate::commands::default::DefaultCommand;
use crate::commands::fuelup::FuelupCommand;
use crate::commands::toolchain::ToolchainCommand;

#[derive(Debug, Parser)]
#[clap(name = "fuelup", about = "Fuel Toolchain Manager", version)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Parser)]
enum Commands {
    /// Check for updates to Fuel toolchains and fuelup
    Check(CheckCommand),
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
}

pub fn fuelup_cli() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Check(command) => check::exec(command),
        Commands::Component(command) => component::exec(command),
        Commands::Default_(command) => default::exec(command),
        Commands::Fuelup(command) => match command {
            FuelupCommand::Update => fuelup::exec(),
        },
        Commands::Show(_command) => show::exec(),
        Commands::Toolchain(command) => toolchain::exec(command),
    }
}
