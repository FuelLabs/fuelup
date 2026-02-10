use crate::fmt::{bold, colored_bold};
use crate::telemetry::{get_telemetry_status, set_telemetry_status, TelemetryStatus};
use anyhow::Result;
use clap::Parser;
use tracing::info;

#[derive(Debug, Parser)]
pub enum TelemetryCommand {
    /// Show the current telemetry status
    Status,
    /// Enable telemetry (opt-in)
    Enable,
    /// Disable telemetry (opt-out)
    Disable,
}

pub fn exec(command: TelemetryCommand) -> Result<()> {
    match command {
        TelemetryCommand::Status => show_status(),
        TelemetryCommand::Enable => enable(),
        TelemetryCommand::Disable => disable(),
    }
}

fn show_status() -> Result<()> {
    match get_telemetry_status()? {
        Some(TelemetryStatus::Enabled) => {
            info!(
                "Telemetry is {}",
                colored_bold(ansiterm::Color::Green, "enabled")
            );
            info!("");
            info!("Telemetry helps improve the Fuel toolchain by collecting usage data.");
            info!("To disable, run: {}", bold("fuelup telemetry disable"));
        }
        Some(TelemetryStatus::Disabled) => {
            info!(
                "Telemetry is {}",
                colored_bold(ansiterm::Color::Yellow, "disabled")
            );
            info!("");
            info!("Telemetry is currently disabled. No usage data will be collected.");
            info!("To enable, run: {}", bold("fuelup telemetry enable"));
        }
        None => {
            info!(
                "Telemetry status is {}",
                colored_bold(ansiterm::Color::Yellow, "not set")
            );
            info!("");
            info!("Telemetry preference has not been set. It will be disabled by default.");
            info!("To enable, run: {}", bold("fuelup telemetry enable"));
            info!("To disable, run: {}", bold("fuelup telemetry disable"));
        }
    }
    Ok(())
}

fn enable() -> Result<()> {
    set_telemetry_status(TelemetryStatus::Enabled)?;
    info!(
        "Telemetry has been {}",
        colored_bold(ansiterm::Color::Green, "enabled")
    );
    info!("");
    info!("Thank you for helping improve the Fuel toolchain!");
    info!(
        "To disable at any time, run: {}",
        bold("fuelup telemetry disable")
    );
    Ok(())
}

fn disable() -> Result<()> {
    set_telemetry_status(TelemetryStatus::Disabled)?;
    info!(
        "Telemetry has been {}",
        colored_bold(ansiterm::Color::Yellow, "disabled")
    );
    info!("");
    info!("Telemetry will no longer collect usage data.");
    info!(
        "To enable at any time, run: {}",
        bold("fuelup telemetry enable")
    );
    Ok(())
}
