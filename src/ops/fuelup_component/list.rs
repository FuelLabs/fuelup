use crate::ops::fuelup_toolchain::install::NIX_CMD;
use crate::{
    commands::component::ListCommand, download::get_latest_version, fmt::bold, toolchain::Toolchain,
};
use anyhow::{bail, Result};
use component::Components;
use semver::Version;
use std::io::Write;
use std::process::Command;
use tracing::info;

fn format_installed_component_info(
    name: &str,
    version: Option<String>,
    version_info: &str,
) -> String {
    if let Some(v) = version {
        format!("  {name} {v} ({version_info})\n")
    } else {
        format!("  {name} : failed getting current version\n")
    }
}

fn format_installable_component_info(name: &str, latest_version: &str) -> String {
    format!("  {name} (latest: {latest_version})\n")
}

fn format_forc_default_plugins(plugin_executables: Vec<String>) -> String {
    format!(
        "    - {}\n",
        plugin_executables
            .iter()
            .filter(|c| *c != component::FORC)
            .map(|s| format!("{s} "))
            .collect::<String>(),
    )
}

const PROFILE_LIST: &[&str; 2] = &["profile", "list"];
pub fn list(_command: ListCommand) -> Result<()> {
    if let Err(err) = Command::new(NIX_CMD).args(PROFILE_LIST).output() {
        bail!("failed to show installed binaries for profile: {err}")
    }

    // let toolchain = Toolchain::from_settings()?;

    // // use write! instead of writeln! here to prevent this from printing first.
    // bold(|s| write!(s, "{}", toolchain.name));

    // let mut installed_components_summary = String::from("\nInstalled:\n");
    // let mut available_components_summary = String::from("Installable:\n");

    // let components = Components::collect_publishables()?;
    // for component in components {
    //     let latest_version = get_latest_version(&component.name).map_or_else(
    //         |_| String::from("failed to get latest version"),
    //         |v| v.to_string(),
    //     );
    //     if toolchain.has_component(&component.name) {
    //         let exec_path = toolchain.bin_path.join(&component.name);

    //         let current_version = if let Ok(o) = std::process::Command::new(exec_path)
    //             .arg("--version")
    //             .output()
    //         {
    //             let output = String::from_utf8_lossy(&o.stdout).into_owned();
    //             output.split_whitespace().last().map_or_else(
    //                 || None,
    //                 |v| Version::parse(v).map_or_else(|_| None, |v| Some(v.to_string())),
    //             )
    //         } else {
    //             None
    //         };

    //         let version_info = match Some(&latest_version) == current_version.as_ref() {
    //             true => "up-to-date".to_string(),
    //             false => format!("latest: {}", &latest_version),
    //         };

    //         installed_components_summary.push_str(&format_installed_component_info(
    //             &component.name,
    //             current_version,
    //             &version_info,
    //         ));

    //         if component.name == component::FORC {
    //             installed_components_summary
    //                 .push_str(&format_forc_default_plugins(component.executables))
    //         }
    //     } else {
    //         available_components_summary.push_str(&format_installable_component_info(
    //             &component.name,
    //             &latest_version,
    //         ));

    //         if component.name == component::FORC {
    //             available_components_summary
    //                 .push_str(&format_forc_default_plugins(component.executables))
    //         }
    //     }
    // }
    // info!(
    //     "{}\n{}",
    //     installed_components_summary, available_components_summary
    // );

    Ok(())
}
