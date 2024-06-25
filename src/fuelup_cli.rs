use crate::commands::{
    check::{self, CheckCommand},
    completions::{self, CompletionsCommand},
    component::{self, ComponentCommand},
    default::{self, DefaultCommand},
    fuelup::{self, FuelupCommand},
    toolchain::{self, ToolchainCommand},
    upgrade::{self, UpgradeCommand},
};
use crate::ops::{fuelup_show, fuelup_update};
use anyhow::Result;
use clap::Parser;

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
    Show,
    /// Updates the distributable toolchains, if already installed
    Update,
    /// Updates fuelup itself, switches to the `latest` channel and updates components in all channels.
    Upgrade(UpgradeCommand),
}

pub fn fuelup_cli() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Check(command) => check::exec(command),
        Commands::Completions(command) => completions::exec(command),
        Commands::Component(command) => component::exec(command),
        Commands::Default_(command) => default::exec(command),
        Commands::Fuelup(command) => match command {
            FuelupCommand::Update(update) => fuelup::update_exec(update.force),
            FuelupCommand::Uninstall(remove) => fuelup::remove_exec(remove.force),
        },
        Commands::Show => fuelup_show::show(),
        Commands::Toolchain(command) => toolchain::exec(command),
        Commands::Update => fuelup_update::update(),
        Commands::Upgrade(command) => upgrade::exec(command.force),
    }
}
