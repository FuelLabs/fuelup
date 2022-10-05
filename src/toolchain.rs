use anyhow::{bail, Context, Result};
use component::{self, Components};
use std::fmt;
use std::fs::{remove_dir_all, remove_file};
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;
use time::Date;
use tracing::{error, info};

use crate::channel;
use crate::constants::DATE_FORMAT;
use crate::download::{download_file_and_unpack, link_to_fuelup, unpack_bins, DownloadCfg};
use crate::ops::fuelup_self::self_update;
use crate::path::{
    ensure_dir_exists, fuelup_bin, fuelup_bin_dir, fuelup_dir, fuelup_tmp_dir, settings_file,
    toolchain_bin_dir, toolchain_dir,
};
use crate::settings::SettingsFile;
use crate::target_triple::TargetTriple;

pub const RESERVED_TOOLCHAIN_NAMES: &[&str] = &[
    channel::LATEST,
    channel::BETA,
    channel::NIGHTLY,
    channel::STABLE,
];

#[derive(Debug, Eq, PartialEq)]
pub enum DistToolchainName {
    Latest,
    Nightly,
}

impl fmt::Display for DistToolchainName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DistToolchainName::Latest => write!(f, "{}", channel::LATEST),
            DistToolchainName::Nightly => write!(f, "{}", channel::NIGHTLY),
        }
    }
}

impl FromStr for DistToolchainName {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        match s {
            channel::LATEST => Ok(Self::Latest),
            channel::NIGHTLY => Ok(Self::Nightly),
            _ => bail!("Unknown name for toolchain: {}", s),
        }
    }
}

#[derive(Debug)]
pub struct OfficialToolchainDescription {
    pub name: DistToolchainName,
    pub date: Option<Date>,
    pub target: Option<TargetTriple>,
}

fn parse_metadata(metadata: String) -> Result<(Option<Date>, Option<TargetTriple>)> {
    let (first, second) = metadata.split_at(std::cmp::min(10, metadata.len()));

    match Date::parse(first, DATE_FORMAT) {
        Ok(d) => match TargetTriple::new(second.trim_start_matches('-')) {
            Ok(t) => Ok((Some(d), Some(t))),
            Err(_) => Ok((Some(d), None)),
        },
        Err(_) => match TargetTriple::new(&metadata) {
            Ok(t) => Ok((None, Some(t))),
            Err(_) => bail!("Failed to parse date or target"),
        },
    }
}

impl fmt::Display for OfficialToolchainDescription {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let target = TargetTriple::from_host().unwrap_or_default();
        match self.date {
            Some(d) => write!(f, "{}-{}-{}", self.name, d, target),
            None => write!(f, "{}-{}", self.name, target),
        }
    }
}

impl FromStr for OfficialToolchainDescription {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        if s.ends_with('-') && s.matches('-').count() == 1 {
            bail!("Invalid official toolchain name '{}'", s);
        }

        let (name, metadata) = s.split_once('-').unwrap_or((s, ""));

        if metadata.is_empty() {
            Ok(Self {
                name: DistToolchainName::from_str(name)?,
                date: None,
                target: None,
            })
        } else if let Ok((_, _)) = parse_metadata(metadata.to_string()) {
            bail!(
                "You may not specify a date or target for official toolchain name '{}' yet.",
                name
            );

            // TODO: uncomment once specifying date and target is supported
            // Ok(Self {
            //     name: DistToolchainName::from_str(name)?,
            //     date,
            //     target,
            // })
        } else {
            bail!("Invalid official toolchain name '{}'", s);
        }
    }
}

#[derive(Debug)]
pub struct Toolchain {
    pub name: String,
    pub path: PathBuf,
    pub bin_path: PathBuf,
}

impl Toolchain {
    pub fn new(name: &str) -> Result<Self> {
        let target = TargetTriple::from_host()?;
        let toolchain = format!("{}-{}", name, target);
        Ok(Self {
            name: toolchain.clone(),
            path: toolchain_dir(&toolchain),
            bin_path: toolchain_bin_dir(&toolchain),
        })
    }

    pub fn from(toolchain: &str) -> Result<Self> {
        Ok(Self {
            name: toolchain.to_string(),
            path: toolchain_dir(toolchain),
            bin_path: toolchain_bin_dir(toolchain),
        })
    }

    pub fn from_settings() -> Result<Self> {
        let settings = SettingsFile::new(settings_file());
        let toolchain_name = match settings.with(|s| Ok(s.default_toolchain.clone()))? {
            Some(t) => t,
            None => {
                bail!("No default toolchain detected. Please install or create a toolchain first.")
            }
        };

        Ok(Self {
            name: toolchain_name.clone(),
            path: toolchain_dir(&toolchain_name),
            bin_path: toolchain_bin_dir(&toolchain_name),
        })
    }

    pub fn is_official(&self) -> bool {
        RESERVED_TOOLCHAIN_NAMES.contains(&self.name.split_once('-').unwrap_or((&self.name, "")).0)
    }

    pub fn exists(&self) -> bool {
        self.path.exists() && self.path.is_dir()
    }

    pub fn has_component(&self, component: &str) -> bool {
        let executables = &Components::collect()
            .expect("Failed to collect components")
            .component[component]
            .executables;

        executables.iter().all(|e| self.bin_path.join(e).is_file())
    }

    fn can_remove(&self, component: &str) -> bool {
        // Published components are the ones downloadable, and hence removable.
        Components::contains_published(component)
    }

    pub fn add_component(&self, download_cfg: DownloadCfg) -> Result<DownloadCfg> {
        // Pre-install checks: ensuring toolchain dir, fuelup bin dir, and fuelup exist
        ensure_dir_exists(&self.bin_path)?;

        let fuelup_bin_dir = fuelup_bin_dir();
        ensure_dir_exists(&fuelup_bin_dir)?;

        if !fuelup_bin().is_file() {
            info!("fuelup not found - attempting to self update");
            match self_update() {
                Ok(()) => info!("fuelup installed."),
                Err(e) => bail!("Could not install fuelup: {}", e),
            };
        }

        info!(
            "Adding component {} v{} to '{}'",
            &download_cfg.name, &download_cfg.version, self.name
        );

        if let Err(e) = download_file_and_unpack(&download_cfg, &self.bin_path) {
            bail!(
                "Could not add component {}({}): {}",
                &download_cfg.name,
                &download_cfg.version,
                e
            )
        };

        if let Ok(downloaded) = unpack_bins(&self.bin_path, &fuelup_bin_dir) {
            link_to_fuelup(downloaded)?;

            // Little hack here to download core and std lib upon installing `forc`
            if download_cfg.name == component::FORC {
                let fuelup_tmp_dir = fuelup_tmp_dir();
                ensure_dir_exists(&fuelup_tmp_dir)?;
                let forc_bin_path = self.bin_path.join(component::FORC);
                let temp_project = tempfile::Builder::new()
                    .prefix("temp-project")
                    .tempdir_in(fuelup_tmp_dir)?;
                let temp_project_path = temp_project.path().to_str().unwrap();
                if Command::new(&forc_bin_path)
                    .args(["init", "--path", temp_project_path])
                    .stdout(std::process::Stdio::null())
                    .status()
                    .is_ok()
                {
                    info!("Fetching core forc dependencies");
                    if Command::new(forc_bin_path)
                        .args(["check", "--path", temp_project_path])
                        .stdout(std::process::Stdio::null())
                        .status()
                        .is_err()
                    {
                        error!("Failed to fetch core forc dependencies");
                    };
                };
            }
        };

        info!(
            "Installed {} v{} for toolchain '{}'",
            download_cfg.name, download_cfg.version, self.name
        );

        Ok(download_cfg)
    }

    fn remove_executables(&self, component: &str) -> Result<()> {
        let executables = &Components::collect().unwrap().component[component].executables;
        for executable in executables {
            remove_file(self.bin_path.join(executable))
                .with_context(|| format!("failed to remove executable '{}'", executable))?;
        }
        Ok(())
    }

    pub fn remove_component(&self, component: &str) -> Result<()> {
        if self.can_remove(component) {
            if self.has_component(component) {
                info!("Removing '{}' from toolchain '{}'", component, self.name);
                match self.remove_executables(component) {
                    Ok(_) => info!("'{}' removed from toolchain '{}'", component, self.name),
                    Err(e) => error!(
                        "Failed to remove '{}' from toolchain '{}': {}",
                        component, self.name, e
                    ),
                };
            } else {
                info!("'{}' not found in toolchain '{}'", component, self.name);
            }
        } else {
            info!("'{}' is not a removable component", component);
        }

        Ok(())
    }

    pub fn uninstall_self(&self) -> Result<()> {
        if self.exists() {
            remove_dir_all(self.path.clone())?
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DATE: &str = "2022-08-29";
    const DATE_TARGET_APPLE: &str = "2022-08-29-x86_64-apple-darwin";

    const TARGET_X86_APPLE: &str = "x86_64-apple-darwin";
    const TARGET_ARM_APPLE: &str = "aarch64-apple-darwin";
    const TARGET_X86_LINUX: &str = "x86_64-unknown-linux-gnu";
    const TARGET_ARM_LINUX: &str = "aarch64-unknown-linux-gnu";

    const NIGHTLY_DATE: &str = "nightly-2022-08-29";

    #[test]
    fn test_parse_name() -> Result<()> {
        for name in [channel::LATEST, channel::NIGHTLY] {
            let desc = OfficialToolchainDescription::from_str(name)?;
            assert_eq!(desc.name, DistToolchainName::from_str(name).unwrap());
            assert_eq!(desc.date, None);
            assert_eq!(desc.target, None);
        }

        Ok(())
    }

    #[test]
    fn test_parse_name_should_fail() -> Result<()> {
        let inputs = ["latest-2", "nightly-toolchain"];
        for name in inputs {
            assert!(OfficialToolchainDescription::from_str(name).is_err());
        }

        Ok(())
    }

    #[test]
    fn test_parse_nightly_date() -> Result<()> {
        assert!(OfficialToolchainDescription::from_str(NIGHTLY_DATE).is_err());

        // TODO: uncomment once specifying date and target is supporting
        //assert_eq!(desc.name, DistToolchainName::from_str("nightly").unwrap());
        //assert_eq!(desc.date.unwrap().to_string(), DATE);
        //assert_eq!(desc.target, None);

        Ok(())
    }

    #[test]
    fn test_parse_nightly_date_target() -> Result<()> {
        for target in [
            TARGET_ARM_APPLE,
            TARGET_ARM_LINUX,
            TARGET_X86_APPLE,
            TARGET_X86_LINUX,
        ] {
            let input = channel::NIGHTLY.to_owned() + "-" + DATE + "-" + target;
            assert!(OfficialToolchainDescription::from_str(&input).is_err());
            // TODO: uncomment once specifying date and target is supporting
            //   assert_eq!(
            //       desc.name,
            //       DistToolchainName::from_str(channel::NIGHTLY).unwrap()
            //   );
            //   assert_eq!(desc.date.unwrap().to_string(), DATE);
            //   assert_eq!(desc.target.unwrap().to_string(), target);
        }

        Ok(())
    }

    #[test]
    fn test_parse_name_target() -> Result<()> {
        for target in [
            TARGET_ARM_APPLE,
            TARGET_ARM_LINUX,
            TARGET_X86_APPLE,
            TARGET_X86_LINUX,
        ] {
            for name in [channel::LATEST, channel::NIGHTLY] {
                let toolchain = name.to_owned() + "-" + target;
                assert!(OfficialToolchainDescription::from_str(&toolchain).is_err());

                // TODO: uncomment once specifying date and target is supporting
                // assert_eq!(desc.name, DistToolchainName::from_str(name).unwrap());
                // assert!(desc.date.is_none());
                // assert_eq!(desc.target.unwrap().to_string(), target);
            }
        }

        Ok(())
    }

    #[test]
    fn test_parse_metadata_date() -> Result<()> {
        let (date, _) = parse_metadata(DATE.to_string())?;
        assert_eq!(DATE, date.unwrap().to_string());
        Ok(())
    }

    #[test]
    fn test_parse_metadata_date_target() -> Result<()> {
        let (date, target) = parse_metadata(DATE_TARGET_APPLE.to_string())?;
        assert_eq!(DATE, date.unwrap().to_string());
        assert_eq!(TARGET_X86_APPLE, target.unwrap().to_string());
        Ok(())
    }

    #[test]
    fn test_parse_metadata_should_fail() -> Result<()> {
        const INPUTS: &[&str] = &["2022", "2022-8-1", "2022-8", "2022-8-x86_64-apple-darwin"];
        for input in INPUTS {
            assert!(parse_metadata(input.to_string()).is_err());
        }
        Ok(())
    }
}
