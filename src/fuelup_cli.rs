use crate::commands::{
    check::{self, CheckCommand},
    completions::{self, CompletionsCommand},
    component::{self, ComponentCommand},
    default::{self, DefaultCommand},
    fuelup::{self, FuelupCommand},
    toolchain::{self, ToolchainCommand},
    upgrade::{self, UpgradeCommand},
};
use anyhow::{bail, Context, Result};
use clap::Parser;
use crate::fmt::ask_user_yes_no_question;
use crate::ops::{fuelup_show, fuelup_toolchain, fuelup_update};
use crate::toolchain::{DistToolchainDescription, Toolchain};
use crate::toolchain_override::ToolchainOverride;
use std::str::FromStr;
use tracing::info;

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

    if let Some(toolchain_override) = ToolchainOverride::from_project_root() {
        let override_path = toolchain_override.cfg.toolchain.channel.to_string();
        let toolchain = match DistToolchainDescription::from_str(&override_path) {
            Ok(desc) => Toolchain::from_path(&desc.to_string()),
            Err(_) => Toolchain::from_path(&override_path),
        };

        info!("Using override toolchain '{}'", &toolchain.name);

        if !toolchain.exists() {
            match cli.command {
                Commands::Toolchain(_) => {
                    // User is managing their toolchains, so we fall through
                }
                _ => {
                    let should_install = ask_user_yes_no_question(
                        "Override toolchain is not installed. Do you want to install it now?",
                    )
                    .context("Console I/O")?;

                    if should_install {
                        fuelup_toolchain::install::install(toolchain::InstallCommand {
                            name: toolchain.name,
                        })?;
                    } else {
                        bail!(
                            "Override toolchain is not installed. Please run: 'fuelup toolchain install {}'",
                            &toolchain.name,
                        )
                    }
                }
            }
        }
    }

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
