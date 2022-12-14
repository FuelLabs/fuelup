use anyhow::Result;
use std::ffi::OsString;
use std::io::{Error, ErrorKind};
use std::os::unix::prelude::CommandExt;
use std::process::{Command, ExitCode, Stdio};
use std::str::FromStr;
use std::{env, io};

use crate::file;
use crate::path::{get_fuel_toolchain, toolchains_dir};
use crate::toolchain::{DistToolchainDescription, DistToolchainName, Toolchain};
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

fn direct_proxy(proc_name: &str, args: &[OsString], toolchain: &Toolchain) -> io::Result<ExitCode> {
    let mut toolchain_override: Option<ToolchainOverride> = None;

    if let Some(fuel_toolchain_toml_file) = get_fuel_toolchain() {
        let fuel_toolchain_toml =
            file::read_file("fuel-toolchain", &fuel_toolchain_toml_file).unwrap();

        toolchain_override = ToolchainOverride::parse(&fuel_toolchain_toml).ok();
    }

    let (bin_path, toolchain_name) = match toolchain_override {
        Some(to) => {
            let name = match DistToolchainDescription::from_str(&to.toolchain.name) {
                Ok(n) => n.to_string(),
                Err(_) => to.toolchain.name,
            };
            (
                toolchains_dir()
                    .join(name.to_string())
                    .join("bin")
                    .join(proc_name),
                name.to_string(),
            )
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
