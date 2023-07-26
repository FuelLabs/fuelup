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

fn _format_installed_component_info(
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

fn _format_installable_component_info(name: &str, latest_version: &str) -> String {
    format!("  {name} (latest: {latest_version})\n")
}

fn _format_forc_default_plugins(plugin_executables: Vec<String>) -> String {
    format!(
        "    - {}\n",
        plugin_executables
            .iter()
            .filter(|c| *c != component::FORC)
            .map(|s| format!("{s} "))
            .collect::<String>(),
    )
}

// todo: format output for listed components, currently shows toolchains as well which
// needs to be changed
const PROFILE_LIST: &[&str; 2] = &["profile", "list"];
pub fn list(_command: ListCommand) -> Result<()> {
    match Command::new(NIX_CMD).args(PROFILE_LIST).output() {
        Ok(output) => {
            let output_strs = std::str::from_utf8(&output.stdout)?
                .split(' ')
                .filter(|s| s.contains("/nix/store"))
                .map(|s| s.trim())
                .collect::<Vec<&str>>();
            for s in output_strs {
                info!("{:#?}", s)
            }
        }
        Err(err) => bail!("failed to show installed binaries for profile: {err}"),
    }

    Ok(())
}
