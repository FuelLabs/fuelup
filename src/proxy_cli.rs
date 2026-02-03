use crate::{
    download::DownloadCfg,
    target_triple::TargetTriple,
    telemetry::is_telemetry_enabled,
    toolchain::{DistToolchainDescription, Toolchain},
    toolchain_override::{ComponentSpec, ToolchainOverride},
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

            // Check if a specific component is declared in the override
            if let Some(spec) = to.get_component_spec(component_name) {
                match spec {
                    ComponentSpec::Version(version) => {
                        // For version specs, ensure the component is linked into the toolchain bin
                        let download_cfg = DownloadCfg::new(
                            component_name,
                            TargetTriple::from_component(component_name)?,
                            Some(version.clone()),
                        )?;
                        toolchain.add_component(download_cfg)?;

                        (toolchain.bin_path.join(proc_name), description.to_string())
                    }
                    ComponentSpec::Path(_) => {
                        // For path specs, validate this specific component and use the resolved path
                        spec.validate_binary(&to.base_dir())?;

                        let local_path = spec.resolve_path(&to.base_dir()).ok_or_else(|| {
                            anyhow::Error::msg(format!(
                                "Failed to resolve local path for component '{component_name}'"
                            ))
                        })?;

                        (local_path, description.to_string())
                    }
                }
            } else {
                (toolchain.bin_path.join(proc_name), description.to_string())
            }
        }
        None => (toolchain.bin_path.join(proc_name), toolchain.name.clone()),
    };

    let mut cmd = Command::new(bin_path);

    cmd.args(args);
    cmd.stdin(Stdio::inherit());

    // Set FUELUP_NO_TELEMETRY based on user's opt-in preference
    // FUELUP_NO_TELEMETRY disables telemetry, so we set it when user has opted out
    if !is_telemetry_enabled() {
        cmd.env("FUELUP_NO_TELEMETRY", "1");
    }

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
