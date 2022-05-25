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
    let format = tracing_subscriber::fmt::format()
        .without_time()
        .with_level(false)
        .with_target(false);

    tracing_subscriber::fmt().event_format(format).init();

    let cli = Cli::parse();

    match cli.command {
        Fuelup::Install(_command) => install(),
    }
}
