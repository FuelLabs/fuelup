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

pub fn check() -> Result<()> {
    let cfg = Config::from_env()?;
    let toolchains = cfg.list_toolchains()?;

    for toolchain in toolchains {
        if let Ok(toolchain) = Toolchain::from(&toolchain) {
            print_boldln(&toolchain.name)?;
            for component in [component::FORC, component::FUEL_CORE] {
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
                            print_bold(&format!(
                                "  {} (plugins: forc-explore, forc-fmt, forc-lsp) - ",
                                component
                            ))?;
                        } else {
                            print_bold(&format!("  {} - ", component))?;
                        }

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

            print_bold(&format!("{} - ", component::FUELUP))?;
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
