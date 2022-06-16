use anyhow::Result;
use std::ffi::OsString;
use std::os::unix::prelude::CommandExt;
use std::process::{Command, ExitCode, Stdio};
use std::{env, io};

use crate::component;
use crate::path::settings_file;
use crate::settings::SettingsFile;
use crate::toolchain::Toolchain;

/// Runs forc or fuel-core in proxy mode
pub fn proxy_run(arg0: &str) -> Result<ExitCode> {
    let cmd_args: Vec<_> = env::args_os().skip(1).collect();
    let settings_file = SettingsFile::new(settings_file());
    let toolchain =
        settings_file.with(|s| Toolchain::from_settings(&s.default_toolchain.clone().unwrap()))?;

    if !cmd_args.is_empty()
        && component::SUPPORTED_PLUGINS
            .contains(&cmd_args[0].to_str().expect("Failed to parse cmd args"))
    {
        let plugin = &format!("{}-{}", arg0, &cmd_args[0].to_string_lossy());
        direct_proxy(plugin, &cmd_args[1..], toolchain)?;
    } else {
        direct_proxy(arg0, &cmd_args, toolchain)?;
    }

    Ok(ExitCode::SUCCESS)
}

fn direct_proxy(proc_name: &str, args: &[OsString], toolchain: Toolchain) -> io::Result<ExitCode> {
    let bin_path = toolchain.path.join(proc_name);
    let mut cmd = Command::new(bin_path);

    cmd.args(args);
    cmd.stdin(Stdio::inherit());

    return exec(&mut cmd);

    fn exec(cmd: &mut Command) -> io::Result<ExitCode> {
        Err(cmd.exec())
    }
}
