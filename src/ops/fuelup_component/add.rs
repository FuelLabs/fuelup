use std::str::FromStr;

use anyhow::{bail, Result};
use semver::Version;

use crate::{
    commands::component::AddCommand,
    download::{target_from_name, DownloadCfg},
    toolchain::{DistToolchainName, Toolchain},
};

pub fn add(command: AddCommand) -> Result<()> {
    let AddCommand {
        maybe_versioned_component,
    } = command;

    let toolchain = Toolchain::from_settings()?;
    if toolchain.has_component(&maybe_versioned_component) {
        println!(
            "{} already exists; replacing {} in toolchain {}",
            &maybe_versioned_component,
            toolchain.has_component(&maybe_versioned_component),
            toolchain.name
        );
    }

    let (component, version): (&str, Option<Version>) =
        match maybe_versioned_component.split_once('@') {
            Some(t) => {
                if let Ok(toolchain) = DistToolchainName::from_str(&toolchain.name) {
                    bail!(
                    "You cannot specify versions of components to add to official toolchain '{}'",
                    toolchain
                    )
                };
                (t.0, Some(Version::from_str(t.1)?))
            }
            None => (&maybe_versioned_component, None),
        };
    println!("adding component {} to {}", component, &toolchain.name);

    let download_cfg = DownloadCfg::new(component, Some(target_from_name(component)?), version)?;
    toolchain.add_component(download_cfg)?;

    Ok(())
}
