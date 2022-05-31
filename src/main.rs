use anyhow::Result;
use clap::Parser;
use fuelup::commands::install;

use fuelup::commands::fuelup::{self_update, FuelupCommand};
use fuelup::commands::install::InstallCommand;

#[derive(Debug, Parser)]
#[clap(name = "fuelup", about = "Fuel Toolchain Manager", version)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Parser)]
enum Commands {
    /// Installs the latest Fuel toolchain.
    Install(InstallCommand),
    /// Manage your fuelup installation.
    #[clap(name = "self", subcommand)]
    Fuelup(FuelupCommand),
}

fn main() -> Result<()> {
    let format = tracing_subscriber::fmt::format()
        .without_time()
        .with_level(false)
        .with_target(false);

    tracing_subscriber::fmt().event_format(format).init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Install(_command) => install::install(),
        Commands::Fuelup(command) => match command {
            FuelupCommand::Update => self_update(),
        },
    }
}
