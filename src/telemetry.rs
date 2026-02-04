use crate::path::fuelup_dir;
use anyhow::{anyhow, Result};
use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;

const TELEMETRY_OPT_IN_FILE: &str = ".telemetry_opt_in";

/// Get the path to the telemetry opt-in file
pub fn telemetry_opt_in_file() -> PathBuf {
    fuelup_dir().join(TELEMETRY_OPT_IN_FILE)
}

/// Represents the user's telemetry preference
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TelemetryStatus {
    /// User has opted in to telemetry
    Enabled,
    /// User has opted out of telemetry
    Disabled,
}

impl TelemetryStatus {
    /// Convert to boolean (true = enabled, false = disabled)
    pub fn is_enabled(&self) -> bool {
        matches!(self, TelemetryStatus::Enabled)
    }

    /// Convert from boolean
    pub fn from_bool(enabled: bool) -> Self {
        if enabled {
            TelemetryStatus::Enabled
        } else {
            TelemetryStatus::Disabled
        }
    }

    /// Convert to string for storage
    pub fn to_storage_string(&self) -> &'static str {
        match self {
            TelemetryStatus::Enabled => "1",
            TelemetryStatus::Disabled => "0",
        }
    }

    /// Parse from storage string
    pub fn from_storage_string(s: &str) -> Result<Self> {
        match s.trim() {
            "1" | "true" | "enabled" | "yes" | "y" => Ok(TelemetryStatus::Enabled),
            "0" | "false" | "disabled" | "no" | "n" => Ok(TelemetryStatus::Disabled),
            _ => Err(anyhow!("Invalid telemetry status: {}", s)),
        }
    }
}

/// Get the user's telemetry preference from the opt-in file
///
/// Returns `None` if the file doesn't exist (user hasn't been prompted yet)
pub fn get_telemetry_status() -> Result<Option<TelemetryStatus>> {
    let path = telemetry_opt_in_file();

    if !path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&path)?;
    let status = TelemetryStatus::from_storage_string(&content)?;

    Ok(Some(status))
}

/// Set the user's telemetry preference
pub fn set_telemetry_status(status: TelemetryStatus) -> Result<()> {
    let path = telemetry_opt_in_file();

    // Ensure the parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&path, status.to_storage_string())?;
    Ok(())
}

/// Check if telemetry should be enabled based on the opt-in file
///
/// Returns `true` if telemetry is enabled, `false` otherwise.
/// If the file doesn't exist, returns `false` (disabled by default).
pub fn is_telemetry_enabled() -> bool {
    get_telemetry_status()
        .ok()
        .flatten()
        .map(|s| s.is_enabled())
        .unwrap_or(false)
}

/// Prompt the user for telemetry opt-in if they haven't been asked yet
///
/// This function checks if the telemetry opt-in file exists. If not, it prompts
/// the user and saves their choice. Returns `Ok(true)` if prompted, `Ok(false)`
/// if the file already exists (no prompt needed), or `Err` if something fails.
///
/// The prompt will only be shown in an interactive terminal to avoid blocking
/// scripts or automated workflows.
pub fn prompt_telemetry_if_needed() -> Result<bool> {
    // If the file already exists, no need to prompt
    if telemetry_opt_in_file().exists() {
        return Ok(false);
    }

    // Only prompt if we're in an interactive terminal
    // This prevents blocking scripts or CI/CD pipelines
    if !std::io::stdin().is_terminal() {
        // Default to disabled for non-interactive environments
        set_telemetry_status(TelemetryStatus::Disabled)?;
        return Ok(false);
    }

    // Ensure the fuelup directory exists
    if let Some(parent) = telemetry_opt_in_file().parent() {
        fs::create_dir_all(parent)?;
    }

    // Prompt the user
    println!();
    println!("Telemetry helps improve the Fuel toolchain by collecting anonymous usage data.");
    println!("You can change this setting at any time by running:");
    println!("  fuelup telemetry enable   # to enable telemetry");
    println!("  fuelup telemetry disable  # to disable telemetry");
    println!();

    // Read user input
    print!("Would you like to enable telemetry? (Y/n): ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    match input.trim().to_lowercase().as_str() {
        "n" | "no" => {
            set_telemetry_status(TelemetryStatus::Disabled)?;
            println!();
            println!("Telemetry disabled. You can enable it later with 'fuelup telemetry enable'.");
            println!();
        }
        _ => {
            set_telemetry_status(TelemetryStatus::Enabled)?;
            println!();
            println!("Telemetry enabled. Thank you for helping improve the Fuel toolchain!");
            println!("You can disable it at any time with 'fuelup telemetry disable'.");
            println!();
        }
    };

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_string_conversion() {
        assert_eq!(TelemetryStatus::Enabled.to_storage_string(), "1");
        assert_eq!(TelemetryStatus::Disabled.to_storage_string(), "0");
    }

    #[test]
    fn test_from_storage_string() {
        assert_eq!(
            TelemetryStatus::from_storage_string("1").unwrap(),
            TelemetryStatus::Enabled
        );
        assert_eq!(
            TelemetryStatus::from_storage_string("true").unwrap(),
            TelemetryStatus::Enabled
        );
        assert_eq!(
            TelemetryStatus::from_storage_string("0").unwrap(),
            TelemetryStatus::Disabled
        );
        assert_eq!(
            TelemetryStatus::from_storage_string("false").unwrap(),
            TelemetryStatus::Disabled
        );
        assert!(TelemetryStatus::from_storage_string("invalid").is_err());
    }

    #[test]
    fn test_is_enabled() {
        assert!(TelemetryStatus::Enabled.is_enabled());
        assert!(!TelemetryStatus::Disabled.is_enabled());
    }

    #[test]
    fn test_from_bool() {
        assert_eq!(TelemetryStatus::from_bool(true), TelemetryStatus::Enabled);
        assert_eq!(TelemetryStatus::from_bool(false), TelemetryStatus::Disabled);
    }
}
