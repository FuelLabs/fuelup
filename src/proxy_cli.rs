use crate::{
    download::DownloadCfg,
    store::Store,
    target_triple::TargetTriple,
    toolchain::{DistToolchainDescription, Toolchain},
    toolchain_override::ToolchainOverride,
};
use anyhow::Result;
use component::Components;
use std::{
    env,
    ffi::OsString,
    io::{self, Error, ErrorKind},
    os::unix::prelude::CommandExt,
    process::{Command, ExitCode, Stdio},
    str::FromStr,
};

/// Runs forc or fuel-core in proxy mode
pub fn proxy_run(arg0: &str) -> Result<ExitCode> {
    let cmd_args: Vec<_> = env::args_os().skip(1).collect();
    let toolchain = Toolchain::from_settings()?;

    if let Some(first_arg) = cmd_args.first() {
        let plugin = format!("{}-{}", arg0, first_arg.to_string_lossy());
        if Components::collect_plugin_executables()?.contains(&plugin) {
            direct_proxy(&plugin, cmd_args.get(1..).unwrap_or_default(), &toolchain)?;
        }
    }

    direct_proxy(arg0, &cmd_args, &toolchain)?;
    Ok(ExitCode::SUCCESS)
}

fn direct_proxy(proc_name: &str, args: &[OsString], toolchain: &Toolchain) -> Result<ExitCode> {
    let toolchain_override: Option<ToolchainOverride> = ToolchainOverride::from_project_root();

    let (bin_path, toolchain_name) = match toolchain_override {
        Some(to) => {
            // unwrap() is safe here since we try DistToolchainDescription::from_str()
            // when deserializing from the toml.
            let description =
                DistToolchainDescription::from_str(&to.cfg.toolchain.channel.to_string()).unwrap();
            let toolchain = Toolchain::from_path(&description.to_string());

            // Install the entire toolchain declared in [toolchain] if it does not exist.
            toolchain.install_if_nonexistent(&description)?;

            // Plugins distributed by forc have to be handled a little differently,
            // if one of them is called we want to check for 'forc' instead.
            let component_name = if Components::is_distributed_by_forc(proc_name) {
                component::FORC
            } else {
                proc_name
            };

            // If a specific version is declared, we want to call it from the store and not from the toolchain directory.
            if let Some(version) = to.get_component_version(component_name) {
                let store = Store::from_env()?;

                if !store.has_component(component_name, version) {
                    let download_cfg = DownloadCfg::new(
                        component_name,
                        TargetTriple::from_component(component_name)?,
                        Some(version.clone()),
                    )?;
                    // Install components within [components] that are declared but missing from the store.
                    store.install_component(&download_cfg)?;
                };

                (
                    store
                        .component_dir_path(component_name, version)
                        .join(proc_name),
                    description.to_string(),
                )
            } else {
                (toolchain.bin_path.join(proc_name), description.to_string())
            }
        }
        None => (toolchain.bin_path.join(proc_name), toolchain.name.clone()),
    };

    let mut cmd = Command::new(bin_path);

    cmd.args(args);
    cmd.stdin(Stdio::inherit());

    return exec(&mut cmd, proc_name, &toolchain_name).map_err(anyhow::Error::from);

    fn exec(cmd: &mut Command, proc_name: &str, toolchain_name: &str) -> io::Result<ExitCode> {
        let error = cmd.exec();
        match error.kind() {
            ErrorKind::NotFound => Err(Error::new(
                ErrorKind::NotFound,
                format!(
                    "component '{proc_name}' not found in currently active toolchain '{toolchain_name}'"
                ),
            )),
            _ => Err(error),
        }
    }
}
