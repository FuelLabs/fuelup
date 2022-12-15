use anyhow::Result;
use std::ffi::OsString;
use std::io::{Error, ErrorKind};
use std::os::unix::prelude::CommandExt;
use std::process::{Command, ExitCode, Stdio};
use std::str::FromStr;
use std::{env, io};
use tracing::warn;

use crate::download::DownloadCfg;
use crate::file;
use crate::path::get_fuel_toolchain;
use crate::target_triple::TargetTriple;
use crate::toolchain::{DistToolchainDescription, Toolchain};
use crate::toolchain_override::ToolchainOverride;
use component::Components;

/// Runs forc or fuel-core in proxy mode
pub fn proxy_run(arg0: &str) -> Result<ExitCode> {
    let cmd_args: Vec<_> = env::args_os().skip(1).collect();
    let toolchain = Toolchain::from_settings()?;

    if !cmd_args.is_empty() {
        let plugin = format!("{}-{}", arg0, &cmd_args[0].to_string_lossy());
        if Components::collect_plugin_executables()?.contains(&plugin) {
            direct_proxy(&plugin, &cmd_args[1..], &toolchain)?;
        }
    }

    direct_proxy(arg0, &cmd_args, &toolchain)?;
    Ok(ExitCode::SUCCESS)
}

fn install_from_override(toolchain: &Toolchain, components: &[String], called: &str) -> Result<()> {
    for component in components {
        if !toolchain.has_component(component) {
            let target_triple = TargetTriple::from_component(component)
                .unwrap_or_else(|_| panic!("Failed to create target triple for '{}'", component));
            if let Ok(download_cfg) = DownloadCfg::new(called, target_triple, None) {
                toolchain.add_component(download_cfg).unwrap_or_else(|_| {
                    panic!(
                        "Failed to add component '{}' to toolchain '{}'",
                        component, toolchain.name,
                    )
                });
            }
        }
    }
    Ok(())
}

fn direct_proxy(proc_name: &str, args: &[OsString], toolchain: &Toolchain) -> io::Result<ExitCode> {
    let mut toolchain_override: Option<ToolchainOverride> = None;

    if let Some(fuel_toolchain_toml_file) = get_fuel_toolchain() {
        if let Ok(fuel_toolchain_toml) =
            file::read_file("fuel-toolchain", &fuel_toolchain_toml_file)
        {
            toolchain_override = ToolchainOverride::parse(&fuel_toolchain_toml)
                .map(Option::Some)
                .expect("Failed parsing fuel-toolchain.toml at project root");
        }
    }

    let (bin_path, toolchain_name) = match toolchain_override {
        Some(to) => {
            let name = match DistToolchainDescription::from_str(&to.toolchain.name) {
                Ok(n) => n.to_string(),
                Err(_) => to.toolchain.name,
            };

            let toolchain = Toolchain::from_path(&name)
                .unwrap_or_else(|_| panic!("Failed to create toolchain '{}' from path", &name));
            match to.toolchain.components.as_deref() {
                    Some([]) | None => warn!(
                        "warning: overriding toolchain '{}' in fuel-toolchain.toml does not have any components listed",
                        &name
                    ),

                    Some(components) => {
                        if let Err(e) = install_from_override(&toolchain, components, proc_name) {

                            warn!(
                                "warning: could not install toolchain from fuel-toolchain.toml: {}",
                                e
                            )
                        }
                }
                    };
            (toolchain.bin_path.join(proc_name), name)
        }
        None => (
            toolchain.bin_path.join(proc_name),
            toolchain.name.to_owned(),
        ),
    };

    let mut cmd = Command::new(bin_path);

    cmd.args(args);
    cmd.stdin(Stdio::inherit());

    return exec(&mut cmd, proc_name, &toolchain_name);

    fn exec(cmd: &mut Command, proc_name: &str, toolchain_name: &str) -> io::Result<ExitCode> {
        let error = cmd.exec();
        match error.kind() {
            ErrorKind::NotFound => Err(Error::new(
                ErrorKind::NotFound,
                format!(
                    "component '{}' not found in currently active toolchain '{}'",
                    proc_name, toolchain_name
                ),
            )),
            _ => Err(error),
        }
    }
}
