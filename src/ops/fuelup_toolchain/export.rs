use crate::{
    commands::toolchain::ExportCommand,
    file::{get_bin_version, write_file},
    toolchain::{DistToolchainDescription, DistToolchainName, Toolchain},
    toolchain_override::{Channel, ComponentSpec, OverrideCfg, ToolchainCfg},
};
use anyhow::{bail, Result};
use component::Components;
use std::{collections::HashMap, path::Path, str::FromStr};
use time::OffsetDateTime;
use tracing::{info, warn};

pub fn export(command: ExportCommand) -> Result<()> {
    let output_path = Path::new("fuel-toolchain.toml");
    
    // Check if file exists
    if output_path.exists() && !command.force {
        bail!(
            "fuel-toolchain.toml already exists in the current directory. \
             Use --force to overwrite."
        );
    }
    
    // Get toolchain to export
    let toolchain = match command.name {
        Some(name) => {
            let toolchain = Toolchain::from_path(&name);
            if !toolchain.exists() {
                bail!("Toolchain '{}' not found", name);
            }
            toolchain
        }
        None => Toolchain::from_settings()?,
    };
    
    // Parse toolchain name to get channel info
    let channel = parse_toolchain_channel(&toolchain.name)?;
    
    // Collect installed components
    let components = collect_toolchain_components(&toolchain)?;
    
    // Check for local paths and warn user
    let local_components = check_for_local_paths(&components);
    if !local_components.is_empty() {
        warn!(
            "⚠️  Local paths detected in toolchain. These may not work for other users:\n{}",
            local_components.iter().map(|c| format!("  - {}", c)).collect::<Vec<_>>().join("\n")
        );
    }
    
    // Create override config
    let cfg = OverrideCfg::new(
        ToolchainCfg { channel },
        if components.is_empty() { None } else { Some(components) },
    );
    
    // Write to file
    let toml_content = cfg.to_string_pretty()?;
    write_file(output_path, &toml_content)?;
    
    info!("Exported toolchain '{}' to fuel-toolchain.toml", toolchain.name);
    Ok(())
}

fn parse_toolchain_channel(toolchain_name: &str) -> Result<Channel> {
    // Handle distributable toolchains (e.g., "latest-x86_64-apple-darwin")
    if let Ok(desc) = DistToolchainDescription::from_str(toolchain_name) {
        match desc.name {
            DistToolchainName::Latest | DistToolchainName::Nightly => {
                if desc.date.is_some() {
                    // Handle dated channels (e.g. "latest-2023-01-15")
                    Ok(Channel {
                        name: desc.name.to_string(),
                        date: desc.date,
                    })
                } else {
                    // For latest/nightly without date, use today's date to make it valid
                    let today = OffsetDateTime::now_utc().date();
                    Ok(Channel {
                        name: desc.name.to_string(),
                        date: Some(today),
                    })
                }
            }
            DistToolchainName::Testnet | DistToolchainName::Mainnet => {
                // These are dateless channels
                Ok(Channel {
                    name: desc.name.to_string(),
                    date: None,
                })
            }
        }
    } else {
        // Handle custom toolchains
        Ok(Channel {
            name: toolchain_name.to_string(),
            date: None,
        })
    }
}

fn collect_toolchain_components(toolchain: &Toolchain) -> Result<HashMap<String, ComponentSpec>> {
    let mut components = HashMap::new();
    
    for component in Components::collect_publishables()? {
        if toolchain.has_component(&component.name) {
            let bin_path = toolchain.bin_path.join(&component.name);
            if let Ok(version) = get_bin_version(&bin_path) {
                components.insert(
                    component.name.clone(),
                    ComponentSpec::Version(version),
                );
            }
        }
    }
    
    Ok(components)
}

fn check_for_local_paths(components: &HashMap<String, ComponentSpec>) -> Vec<String> {
    components.iter()
        .filter_map(|(name, spec)| {
            if spec.is_path() {
                Some(name.clone())
            } else {
                None
            }
        })
        .collect()
}