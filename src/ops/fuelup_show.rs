use anyhow::Result;
use component::{self, Components};
use semver::Version;
use std::{io::Write, path::Path};
use tracing::{error, info};

use crate::{
    config::Config,
    fmt::{bold, print_header},
    path::fuelup_dir,
    target_triple::TargetTriple,
    toolchain::Toolchain,
    toolchain_override::ToolchainOverride,
};

fn exec_show_version(component_executable: &Path) -> Result<()> {
    match std::process::Command::new(component_executable)
        .arg("--version")
        .output()
    {
        Ok(o) => {
            let output = String::from_utf8_lossy(&o.stdout).into_owned();
            match output.split_whitespace().last() {
                Some(v) => {
                    let version = Version::parse(v)?;
                    info!(" : {}", version);
                }
                None => {
                    error!(" : Error getting version string");
                }
            };
        }
        Err(e) => {
            print!(" - ");
            if component_executable.exists() {
                error!("execution error - {}", e);
            } else {
                error!("not found");
            }
        }
    }

    Ok(())
}

pub fn show() -> Result<()> {
    bold(|s| write!(s, "Default host: "));
    info!("{}", TargetTriple::from_host()?);

    bold(|s| write!(s, "fuelup home: "));
    info!("{}\n", fuelup_dir().display());

    print_header("installed toolchains");
    let cfg = Config::from_env()?;
    let mut active_toolchain = Toolchain::from_settings()?;

    let to = ToolchainOverride::from_file();

    for toolchain in cfg.list_toolchains()? {
        let mut message = toolchain.clone();
        if toolchain == active_toolchain.name {
            message.push_str(" (default)")
        }

        if let Some(to) = to.as_ref() {
            if toolchain == to.toolchain.name {
                message.push_str(" (override)")
            }
        }
        info!("{}", message)
    }

    if let Some(to) = to.as_ref() {
        active_toolchain = Toolchain::from_path(&to.toolchain.name)?;
    }

    print_header("\nactive toolchain");

    info!("{} (default)", active_toolchain.name);

    for component in Components::collect_exclude_plugins()? {
        bold(|s| write!(s, "  {}", &component.name));
        let component_executable = active_toolchain.bin_path.join(&component.name);
        exec_show_version(component_executable.as_path())?;

        if component.name == component::FORC {
            for plugin in Components::collect_plugins()? {
                bold(|s| write!(s, "    - {}", &plugin.name));
                if !plugin.is_main_executable() {
                    info!("");
                    for executable in plugin.executables.iter() {
                        bold(|s| write!(s, "      - {}", &executable));
                        let plugin_executable = active_toolchain.bin_path.join(executable);
                        exec_show_version(plugin_executable.as_path())?;
                    }
                } else {
                    let plugin_executable = active_toolchain.bin_path.join(&plugin.name);
                    exec_show_version(plugin_executable.as_path())?;
                }
            }
        }
    }

    Ok(())
}
