use anyhow::Result;
use fuelup::download::component;
use fuelup::fuelup_cli;
use fuelup::path::settings_file;
use fuelup::settings::SettingsFile;
use fuelup::toolchain::Toolchain;
use std::ffi::OsString;
use std::os::unix::prelude::CommandExt;
use std::path::PathBuf;
use std::process::{Command, ExitCode, Stdio};
use std::{env, io, panic};

fn is_supported_component(component: &str) -> bool {
    ["forc", "fuel-core", "forc-fmt", "forc-lsp", "forc-explore"].contains(&component)
}

fn is_supported_plugin(plugin: &str) -> bool {
    ["fmt", "lsp", "explore"].contains(&plugin)
}

/// Runs forc or fuel-core in proxy mode
fn proxy_run(arg0: &str) -> Result<ExitCode> {
    let cmd_args: Vec<_> = env::args_os().skip(1).collect();
    let settings_file = SettingsFile::new(settings_file());
    let toolchain =
        settings_file.with(|s| Toolchain::from_settings(&s.default_toolchain.clone().unwrap()))?;

    if !cmd_args.is_empty() && is_supported_plugin(&cmd_args[0].to_string_lossy()) {
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

fn run() -> Result<()> {
    let arg0 = std::env::args().next().map(PathBuf::from);

    let process_name = arg0
        .as_ref()
        .and_then(|a| a.file_stem())
        .and_then(std::ffi::OsStr::to_str)
        .map(String::from);

    match process_name.as_deref() {
        Some(component::FUELUP) => fuelup_cli::fuelup_cli()?,
        Some(n) => {
            if is_supported_component(n) {
                proxy_run(n)?;
            }
        }
        None => panic!("Unknown exe"),
    }
    Ok(())
}

fn main() {
    let format = tracing_subscriber::fmt::format()
        .without_time()
        .with_level(false)
        .with_target(false);

    tracing_subscriber::fmt().event_format(format).init();

    if run().is_err() {
        std::process::exit(1);
    }
}
