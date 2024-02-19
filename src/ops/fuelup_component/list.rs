use crate::{
    commands::component::ListCommand, download::get_latest_version, file::get_bin_version,
    fmt::bold, toolchain::Toolchain,
};
use anyhow::Result;
use component::Components;
use std::fmt::Write;
use tracing::info;

fn format_installed_component_info(
    name: &str,
    version: Option<String>,
    version_info: &str,
) -> String {
    if let Some(v) = version {
        format!("{:>2}{name} {v} ({version_info})\n", "")
    } else {
        format!("{:>2}{name} : failed getting current version\n", "")
    }
}

fn format_installable_component_info(name: &str, latest_version: &str) -> String {
    format!("{:>2}{name} (latest: {latest_version})\n", "")
}

fn format_forc_default_plugins(plugin_executables: Vec<String>) -> String {
    plugin_executables
        .iter()
        .filter(|c| *c != component::FORC)
        .fold(String::new(), |mut output, b| {
            let _ = writeln!(output, "{:>4}- {}", "", b);
            output
        })
}

pub fn list(_command: ListCommand) -> Result<()> {
    let toolchain = Toolchain::from_settings()?;
    let mut installed_components_summary = String::from("\nInstalled:\n");
    let mut available_components_summary = String::from("Installable:\n");

    let components = Components::collect_publishables()?;
    for component in components {
        let latest_version = get_latest_version(&component.name).map_or_else(
            |_| String::from("failed to get latest version"),
            |v| v.to_string(),
        );
        if toolchain.has_component(&component.name) {
            let exec_path = toolchain.bin_path.join(&component.name);
            let current_version = get_bin_version(&exec_path).map(|v| v.to_string());
            let version_info = match Some(&latest_version) == current_version.as_ref() {
                true => "up-to-date".to_string(),
                false => format!("latest: {}", &latest_version),
            };

            installed_components_summary.push_str(&format_installed_component_info(
                &component.name,
                current_version,
                &version_info,
            ));

            if component.name == component::FORC {
                installed_components_summary
                    .push_str(&format_forc_default_plugins(component.executables))
            }
        } else {
            available_components_summary.push_str(&format_installable_component_info(
                &component.name,
                &latest_version,
            ));

            if component.name == component::FORC {
                available_components_summary
                    .push_str(&format_forc_default_plugins(component.executables))
            }
        }
    }
    info!("{}", bold(&toolchain.name));
    info!(
        "{}\n{}",
        installed_components_summary, available_components_summary
    );

    Ok(())
}
