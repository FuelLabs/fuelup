use anyhow::Result;
use clap::Parser;
use dirs::home_dir;
use fuelup::commands::toolchain;
use std::ffi::OsString;
use std::os::unix::prelude::CommandExt;
use std::path::PathBuf;
use std::process::{Command, ExitCode, Stdio};
use std::{env, io, panic};

use fuelup::commands::fuelup::{self_update, FuelupCommand};
use fuelup::commands::toolchain::ToolchainCommand;

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
    #[clap(subcommand, alias = "install")]
    Toolchain(ToolchainCommand),
}

fn is_supported_component(component: &str) -> bool {
    ["forc", "fuel-core", "forc-fmt", "forc-lsp", "forc-explore"].contains(&component)
}

/// Runs forc or fuel-core in proxy mode
fn proxy_run(arg0: &str) -> Result<ExitCode> {
    let cmd_args: Vec<_> = env::args_os().skip(1).collect();

    direct_proxy(arg0, &cmd_args)?;

    Ok(ExitCode::SUCCESS)
}

fn direct_proxy(arg0: &str, args: &[OsString]) -> io::Result<ExitCode> {
    let bin_path = home_dir()
        .unwrap()
        .join(".fuelup/toolchains/latest-x86_64-apple-darwin/bin")
        .join(arg0);
    let mut cmd = Command::new(bin_path);

    cmd.args(args);
    cmd.stdin(Stdio::inherit());

    return exec(&mut cmd);

    fn exec(cmd: &mut Command) -> io::Result<ExitCode> {
        Err(cmd.exec())
    }
}

fn run() -> Result<()> {
    let arg0 = std::env::args().next().map(PathBuf::from);

    let process_name = arg0
        .as_ref()
        .and_then(|a| a.file_stem())
        .and_then(std::ffi::OsStr::to_str)
        .map(String::from);

    match process_name.as_deref() {
        Some("fuelup") => fuelup_cli()?,
        Some(n) => {
            if is_supported_component(n) {
                proxy_run(n)?;
            }
        }
        None => panic!("Unknown exe"),
    }
    Ok(())
}

fn fuelup_cli() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Fuelup(command) => match command {
            FuelupCommand::Update => self_update(),
        },
        Commands::Toolchain(command) => toolchain::exec(command),
    }
}

fn main() {
    let format = tracing_subscriber::fmt::format()
        .without_time()
        .with_level(false)
        .with_target(false);

    tracing_subscriber::fmt().event_format(format).init();

    if let Err(_) = run() {
        std::process::exit(1);
    }
}
