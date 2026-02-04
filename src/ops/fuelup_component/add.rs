use crate::{
    commands::component::AddCommand, download::DownloadCfg, target_triple::TargetTriple,
    toolchain::Toolchain,
};
use anyhow::{bail, Result};
use component::Component;
use semver::Version;
use std::str::FromStr;
use tracing::info;

pub fn add(command: AddCommand) -> Result<()> {
    let AddCommand {
        maybe_versioned_component,
    } = command;

    let toolchain = Toolchain::from_settings()?;
    if toolchain.is_distributed() {
        bail!(
            "Installing specific components is reserved for custom toolchains.
You are currently using '{}'.

You may create a custom toolchain using 'fuelup toolchain new <toolchain>'.",
            toolchain.name
        )
    };

    let (component, version): (&str, Option<Version>) =
        match maybe_versioned_component.split_once('@') {
            Some(t) => {
                let v = match Version::from_str(t.1) {
                    Ok(v) => Some(v),
                    Err(e) => bail!(
                        "Invalid version input '{}' while adding component: {}",
                        t.1,
                        e
                    ),
                };
                (t.0, v)
            }
            None => (&maybe_versioned_component, None),
        };

    if let Some(parent) = Component::parent_component_for_executable(component) {
        bail!(
            "'{}' is an executable bundled with '{}'; please do 'fuelup component add {}' if you would like to install or update it.",
            &maybe_versioned_component,
            parent,
            parent
        );
    }

    if toolchain.has_component(component) {
        info!(
            "{} already exists in toolchain '{}'; replacing existing version with {}{}",
            component,
            toolchain.name,
            component,
            version
                .as_ref()
                .map(|v| format!(" ({})", v))
                .unwrap_or_else(|| " (latest)".to_string())
        );
    }

    let download_cfg =
        DownloadCfg::new(component, TargetTriple::from_component(component)?, version)?;
    toolchain.add_component(download_cfg)?;

    Ok(())
}
