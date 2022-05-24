use anyhow::Result;
use clap::Parser;

use fuelup::commands::install::{install, InstallCommand};

#[derive(Debug, Parser)]
#[clap(name = "fuelup", about = "Fuel Toolchain Manager", version)]
struct Cli {
    #[clap(subcommand)]
    command: Fuelup,
}

#[derive(Debug, Parser)]
enum Fuelup {
    /// Installs the latest forc toolchain.
    Install(InstallCommand),
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Fuelup::Install(_command) => install(),
    }
}
