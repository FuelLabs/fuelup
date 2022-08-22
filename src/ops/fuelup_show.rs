use anyhow::Result;
use semver::Version;
use std::io::Write;

use crate::{
    component::{self, SUPPORTED_PLUGINS},
    config::Config,
    fmt::{bold, print_header},
    path::fuelup_dir,
    target_triple::TargetTriple,
    toolchain::Toolchain,
};

pub fn show() -> Result<()> {
    bold(|s| write!(s, "Default host: "));
    println!("{}", TargetTriple::from_host()?);

    bold(|s| write!(s, "fuelup home: "));
    println!("{}", fuelup_dir().display());
    println!();

    print_header("installed toolchains");
    let cfg = Config::from_env()?;
    let current_toolchain = Toolchain::from_settings()?;

    for toolchain in cfg.list_toolchains()? {
        if toolchain == current_toolchain.name {
            println!("{} (default)", toolchain);
        } else {
            println!("{}", toolchain);
        }
    }

    println!();
    print_header("active toolchain");
    let current_toolchain = Toolchain::from_settings()?;

    println!("{} (default)", current_toolchain.name);
    for component in [component::FORC, component::FUEL_CORE] {
        let component_executable = current_toolchain.path.join(component);

        match std::process::Command::new(&component_executable)
            .arg("--version")
            .output()
        {
            Ok(o) => {
                let output = String::from_utf8_lossy(&o.stdout).into_owned();
                match output.split_whitespace().nth(1) {
                    Some(v) => {
                        let version = Version::parse(v)?;
                        bold(|s| write!(s, "  {}", &component));
                        println!(" : {}", version);
                    }
                    None => {
                        eprintln!("  {} - Error getting version string", component);
                    }
                };
            }
            Err(e) => {
                print!("  ");
                bold(|s| write!(s, "{}", &component));
                print!(" - ");
                if component_executable.exists() {
                    println!("execution error - {}", e);
                } else {
                    println!("not found");
                }
            }
        };

        if component == component::FORC {
            for plugin in SUPPORTED_PLUGINS {
                let plugin_executable = current_toolchain.path.join(&plugin);
                if plugin == &component::FORC_DEPLOY {
                    bold(|s| writeln!(s, "    - forc-client"));
                }
                if plugin == &component::FORC_RUN || plugin == &component::FORC_DEPLOY {
                    print!("  ");
                }
                match std::process::Command::new(&plugin_executable)
                    .arg("--version")
                    .output()
                {
                    Ok(o) => {
                        let output = String::from_utf8_lossy(&o.stdout).into_owned();
                        match output.split_whitespace().nth(1) {
                            Some(v) => {
                                let version = Version::parse(v)?;
                                print!("    - ");
                                bold(|s| write!(s, "{}", plugin));
                                println!(" : {}", version);
                            }
                            None => {
                                eprintln!("    - {} - Error getting version string", plugin);
                            }
                        };
                    }
                    Err(e) => {
                        print!("    - ");
                        bold(|s| write!(s, "{}", plugin));
                        print!(" - ");
                        if plugin_executable.exists() {
                            println!("execution error - {}", e);
                        } else {
                            println!("not found");
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
