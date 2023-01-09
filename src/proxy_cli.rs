use anyhow::Result;
use std::ffi::OsString;
use std::io::{Error, ErrorKind};
use std::os::unix::prelude::CommandExt;
use std::process::{Command, ExitCode, Stdio};
use std::str::FromStr;
use std::{env, io};

use crate::store::Store;
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

fn direct_proxy(proc_name: &str, args: &[OsString], toolchain: &Toolchain) -> io::Result<ExitCode> {
    let toolchain_override: Option<ToolchainOverride> = ToolchainOverride::from_project_root();

    let (bin_path, toolchain_name) = match toolchain_override {
        Some(to) => {
            // unwrap() is safe here since we try DistToolchainDescription::from_str()
            // when deserializing from the toml.
            let description =
                DistToolchainDescription::from_str(&to.cfg.toolchain.channel).unwrap();

            // Since we have a valid DistToolchainDescription above, Toolchain::from_path shouldn't fail.
            let toolchain = Toolchain::from_path(&description.to_string()).unwrap_or_else(|_| {
                panic!("Failed to create toolchain '{}' from path", &description)
            });

            toolchain
                .install_if_nonexistent(&description)
                .unwrap_or_else(|_| panic!("could not install toolchain: '{}'", &description));

            if let Some(version) = to.get_component_version(proc_name) {
                let store = Store::from_env().unwrap();
                if store.has_component(proc_name, version)
                    && store.install_component(proc_name, &to).is_ok()
                {
                    (
                        store.component_dir_path(proc_name, version).join(proc_name),
                        description.to_string(),
                    )
                } else {
                    (toolchain.bin_path.join(proc_name), description.to_string())
                };
            }

            (toolchain.bin_path.join(proc_name), description.to_string())
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
