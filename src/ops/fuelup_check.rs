use crate::{
    colors::{print_bold, print_boldln, print_with_color},
    config::Config,
    toolchain::Toolchain,
};
use anyhow::Result;
use clap::Parser;
use semver::Version;
use termcolor::Color;
use tracing::info;

use crate::{component, download::DownloadCfg};

#[derive(Debug, Parser)]
pub struct CheckCommand {}

pub mod plugin {
    pub const FMT: &str = "fmt";
    pub const LSP: &str = "lsp";
    pub const EXPLORE: &str = "explore";
}

pub fn check() -> Result<()> {
    let cfg = Config::from_env()?;
    let toolchains = cfg.list_toolchains()?;

    for toolchain in toolchains {
        if let Ok(toolchain) = Toolchain::from(&toolchain) {
            print_boldln(&toolchain.name)?;
            for component in [component::FORC, component::FUEL_CORE] {
                let mut latest_version: String = String::new();
                let component_executable = toolchain.path.join(component);

                match std::process::Command::new(component_executable)
                    .arg("--version")
                    .output()
                {
                    Ok(o) => {
                        let version = Version::parse(
                            String::from_utf8_lossy(&o.stdout)
                                .split_whitespace()
                                .collect::<Vec<&str>>()[1],
                        )?;

                        let download_cfg: DownloadCfg = DownloadCfg::new(component, None)?;
                        if component == component::FORC {
                            latest_version = download_cfg.version.to_string();
                        }

                        print_bold(&format!("  {} - ", component))?;
                        println!("{}", version);
                        if version == download_cfg.version {
                            print_with_color("Up to date ", Color::Green)?;
                            println!(": {}", version);
                        } else {
                            print_with_color("Update available ", Color::Yellow)?;
                            println!("{} -> {}", version, download_cfg.version);
                        }
                    }
                    Err(_) => {
                        print_bold(&format!("  {}", component))?;
                        info!(" not found");
                    }
                };

                if component == component::FORC {
                    for plugin in [plugin::FMT, plugin::LSP, plugin::EXPLORE] {
                        let plugin_component = component.to_owned() + "-" + plugin;
                        match std::process::Command::new(&plugin_component)
                            .arg("--version")
                            .output()
                        {
                            Ok(o) => {
                                let version = Version::parse(
                                    String::from_utf8_lossy(&o.stdout)
                                        .split_whitespace()
                                        .collect::<Vec<&str>>()[1],
                                )?;

                                if version == Version::parse(&latest_version)? {
                                    print!("    - {} - ", plugin_component);
                                    print_with_color("Up to date ", Color::Green)
                                        .expect("Internal printing error");
                                    println!(": {}", version);
                                } else {
                                    print!("    - {} - ", plugin_component);
                                    print_with_color("Update available ", Color::Yellow)
                                        .expect("Internal printing error");
                                    println!("{} -> {}", version, latest_version);
                                }
                            }
                            Err(_) => {
                                print_bold(&format!("  {}", &plugin_component))?;
                                info!(" not found");
                            }
                        }
                    }
                }
            }
        }
    }

    match std::process::Command::new(component::FUELUP)
        .arg("--version")
        .output()
    {
        Ok(o) => {
            let download_cfg: DownloadCfg = DownloadCfg::new(component::FUELUP, None)?;
            let version = Version::parse(
                String::from_utf8_lossy(&o.stdout)
                    .split_whitespace()
                    .collect::<Vec<&str>>()[1],
            )?;

            print_bold(&format!("  {} - ", component::FUELUP))?;
            if version == download_cfg.version {
                print_with_color("Up to date ", Color::Green)?;
                println!(": {}", version);
            } else {
                print_with_color("Update available ", Color::Yellow)?;
                println!("{} -> {}", version, download_cfg.version);
            }
        }
        Err(_) => {
            print_bold(&format!("  {}", component::FUELUP))?;
            info!(" not found");
        }
    };
    Ok(())
}
