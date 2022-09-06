use anyhow::Result;
use std::ffi::OsString;
use std::os::unix::prelude::CommandExt;
use std::process::{Command, ExitCode, Stdio};
use std::{env, io};

use crate::toolchain::Toolchain;

/// Runs forc or fuel-core in proxy mode
pub fn proxy_run(arg0: &str) -> Result<ExitCode> {
    let cmd_args: Vec<_> = env::args_os().skip(1).collect();
    let toolchain = Toolchain::from_settings()?;

    direct_proxy(arg0, &cmd_args, toolchain)?;
    Ok(ExitCode::SUCCESS)
}

fn direct_proxy(proc_name: &str, args: &[OsString], toolchain: Toolchain) -> io::Result<ExitCode> {
    let bin_path = toolchain.bin_path.join(proc_name);
    let mut cmd = Command::new(bin_path);

    cmd.args(args);
    cmd.stdin(Stdio::inherit());

    return exec(&mut cmd);

    fn exec(cmd: &mut Command) -> io::Result<ExitCode> {
        Err(cmd.exec())
    }
}
