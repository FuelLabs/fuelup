use anyhow::Result;
use component::Components;
use semver::Version;
use tracing::info;

use crate::{commands::component::ListCommand, download::get_latest_version, toolchain::Toolchain};

fn format_installed_component_info(name: &str, version: &str, version_info: &str) -> String {
    format!("  {name} {version} ({version_info})\n")
}

fn format_installable_component_info(name: &str, latest_version: &str) -> String {
    format!("  {name} (latest: {latest_version})\n")
}

pub fn list(_command: ListCommand) -> Result<()> {
    let toolchain = Toolchain::from_settings()?;

    let mut installed_components_summary = String::from("Installed:\n");
    let mut available_components_summary = String::from("Installable:\n");

    let components = Components::collect_publishables()?;
    for component in components {
        let latest_version = get_latest_version(&component.name)
            .map_or_else(|_| "failed getting version".to_string(), |v| v.to_string());
        if toolchain.has_component(&component.name) {
            let exec_path = toolchain.bin_path.join(&component.name);

            let current_version = if let Ok(o) = std::process::Command::new(exec_path)
                .arg("--version")
                .output()
            {
                let output = String::from_utf8_lossy(&o.stdout).into_owned();
                output.split_whitespace().nth(1).map_or_else(
                    || String::default(),
                    |v| Version::parse(v).map_or_else(|_| String::default(), |v| v.to_string()),
                )
            } else {
                String::default()
            };

            let version_info = match latest_version == current_version {
                true => "up-to-date".to_string(),
                false => format!("latest: {latest_version}"),
            };
            installed_components_summary.push_str(&format_installed_component_info(
                &component.name,
                &format!("v{current_version}"),
                &version_info,
            ));
        } else {
            available_components_summary.push_str(&format_installable_component_info(
                &component.name,
                &latest_version,
            ));
        }
    }

    info!(
        "{}\n{}",
        installed_components_summary, available_components_summary
    );

    Ok(())
}
