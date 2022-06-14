use anyhow::Result;
use clap::Parser;

use crate::commands::{fuelup, toolchain};

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
    /// Manage your fuelup installation.
    #[clap(name = "self", subcommand)]
    Fuelup(FuelupCommand),
    /// Install new toolchains or modify/query installed toolchains
    #[clap(subcommand)]
    Toolchain(ToolchainCommand),
}

pub fn fuelup_cli() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Fuelup(command) => match command {
            FuelupCommand::Update => fuelup::exec(),
        },
        Commands::Toolchain(command) => toolchain::exec(command),
    }
}
