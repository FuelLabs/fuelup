use anyhow::{bail, Context, Result};
use component::{self, Components};
use std::fmt;
use std::fs::{remove_dir_all, remove_file};
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;
use time::Date;
use tracing::{error, info};

use crate::channel::{self, is_beta_toolchain, Channel};
use crate::constants::DATE_FORMAT;
use crate::download::DownloadCfg;
use crate::file::{hard_or_symlink_file, is_executable};
use crate::ops::fuelup_self::self_update;
use crate::path::{
    ensure_dir_exists, fuelup_bin, fuelup_bin_dir, fuelup_tmp_dir, settings_file,
    toolchain_bin_dir, toolchain_dir,
};
use crate::settings::SettingsFile;
use crate::store::Store;
use crate::target_triple::TargetTriple;

pub const RESERVED_TOOLCHAIN_NAMES: &[&str] = &[
    channel::LATEST,
    channel::BETA_1,
    channel::BETA_2,
    channel::BETA_3,
    channel::BETA_4,
    channel::NIGHTLY,
    channel::STABLE,
];

#[derive(Debug, Eq, PartialEq)]
pub enum DistToolchainName {
    Beta1,
    Beta2,
    Beta3,
    Beta4,
    Latest,
    Nightly,
}

impl fmt::Display for DistToolchainName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DistToolchainName::Latest => write!(f, "{}", channel::LATEST),
            DistToolchainName::Nightly => write!(f, "{}", channel::NIGHTLY),
            DistToolchainName::Beta1 => write!(f, "{}", channel::BETA_1),
            DistToolchainName::Beta2 => write!(f, "{}", channel::BETA_2),
            DistToolchainName::Beta3 => write!(f, "{}", channel::BETA_3),
            DistToolchainName::Beta4 => write!(f, "{}", channel::BETA_4),
        }
    }
}

impl FromStr for DistToolchainName {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        match s {
            channel::LATEST => Ok(Self::Latest),
            channel::NIGHTLY => Ok(Self::Nightly),
            channel::BETA_1 => Ok(Self::Beta1),
            channel::BETA_2 => Ok(Self::Beta2),
            channel::BETA_3 => Ok(Self::Beta3),
            channel::BETA_4 => Ok(Self::Beta4),
            _ => bail!("Unknown name for toolchain: {}", s),
        }
    }
}

#[derive(Debug)]
pub struct DistToolchainDescription {
    pub name: DistToolchainName,
    pub date: Option<Date>,
    pub target: Option<TargetTriple>,
}

fn parse_metadata(metadata: String) -> Result<(Option<Date>, Option<TargetTriple>)> {
    if metadata.is_empty() {
        return Ok((None, None));
    }

    let (first, second) = metadata.split_at(std::cmp::min(10, metadata.len()));

    match Date::parse(first, DATE_FORMAT) {
        Ok(d) => {
            if second.is_empty() {
                Ok((Some(d), None))
            } else {
                let target = second.trim_start_matches('-');
                bail!(
                    "You specified target '{}': specifying a target is not supported yet.",
                    target
                );
            }
        }
        Err(_) => match TargetTriple::new(&metadata) {
            Ok(t) => Ok((None, Some(t))),
            Err(_) => bail!("Failed to parse date or target"),
        },
    }
}

impl fmt::Display for DistToolchainDescription {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let target = TargetTriple::from_host().unwrap_or_default();
        match self.date {
            Some(d) => write!(f, "{}-{}-{}", self.name, d, target),
            None => write!(f, "{}-{}", self.name, target),
        }
    }
}

impl FromStr for DistToolchainDescription {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        if s.ends_with('-') && s.matches('-').count() == 1 {
            bail!("Invalid distributable toolchain name '{}'", s);
        }

        let (name, metadata) = s.split_once('-').unwrap_or((s, ""));

        if metadata.is_empty() {
            Ok(Self {
                name: DistToolchainName::from_str(name)?,
                date: None,
                target: TargetTriple::from_host().ok(),
            })
        } else {
            match parse_metadata(metadata.to_string()) {
                Ok((date, target)) => Ok(Self {
                    name: DistToolchainName::from_str(name)?,
                    date,
                    target,
                }),
                Err(e) => {
                    if is_beta_toolchain(s) {
                        Ok(Self {
                            name: DistToolchainName::from_str(s)?,
                            date: None,
                            target: TargetTriple::from_host().ok(),
                        })
                    } else {
                        bail!("Invalid toolchain metadata within input '{}' - {}", s, e)
                    }
                }
            }
        }
    }
}

fn cache_sway_std_libs(forc_bin_path: PathBuf) -> Result<()> {
    let fuelup_tmp_dir = fuelup_tmp_dir();
    ensure_dir_exists(&fuelup_tmp_dir)?;
    let temp_project = tempfile::Builder::new().prefix("temp-project").tempdir()?;
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

    Ok(())
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
        let toolchain = format!("{name}-{target}");
        Ok(Self {
            name: toolchain.clone(),
            path: toolchain_dir(&toolchain),
            bin_path: toolchain_bin_dir(&toolchain),
        })
    }

    pub fn from_path(toolchain: &str) -> Self {
        Self {
            name: toolchain.to_string(),
            path: toolchain_dir(toolchain),
            bin_path: toolchain_bin_dir(toolchain),
        }
    }

    pub fn from_settings() -> Result<Self> {
        let settings = SettingsFile::new(settings_file());

        if settings_file().exists() {
            if let Some(t) = settings.with(|s| Ok(s.default_toolchain.clone()))? {
                return Ok(Self {
                    name: t.clone(),
                    path: toolchain_dir(&t),
                    bin_path: toolchain_bin_dir(&t),
                });
            }
        };

        bail!("No default toolchain detected. Please install or create a toolchain first.")
    }

    pub fn is_distributed(&self) -> bool {
        RESERVED_TOOLCHAIN_NAMES.contains(&self.name.split_once('-').unwrap_or((&self.name, "")).0)
    }

    pub fn exists(&self) -> bool {
        self.path.exists() && self.path.is_dir()
    }

    pub fn has_component(&self, component: &str) -> bool {
        if let Some(component) = Components::collect()
            .expect("Failed to collect components")
            .component
            .get(component)
        {
            component
                .executables
                .iter()
                .all(|e| self.bin_path.join(e).is_file())
        } else {
            false
        }
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

        let fuelup_bin = fuelup_bin();
        if !fuelup_bin.is_file() {
            info!("fuelup not found - attempting to self update");
            match self_update() {
                Ok(()) => info!("fuelup installed."),
                Err(e) => bail!("Could not install fuelup: {}", e),
            };
        }

        let store = Store::from_env()?;

        info!(
            "\nAdding component {} v{} to '{}'",
            &download_cfg.name, &download_cfg.version, self.name
        );

        if !store.has_component(&download_cfg.name, &download_cfg.version)
            || !self.has_component(&download_cfg.name)
        {
            match store.install_component(&download_cfg) {
                Ok(downloaded) => {
                    for bin in downloaded {
                        if is_executable(bin.as_path()) {
                            if let Some(exe_file_name) = bin.file_name() {
                                // Link binary in store -> binary in the toolchain dir
                                hard_or_symlink_file(
                                    bin.as_path(),
                                    &self.bin_path.join(exe_file_name),
                                )?;
                                if !fuelup_bin_dir.join(exe_file_name).exists() {
                                    // Link real 'fuelup' bin -> fake 'fuelup' that acts as
                                    // the installed component in ~/.fuelup/bin, eg. 'forc'
                                    hard_or_symlink_file(
                                        &fuelup_bin,
                                        &fuelup_bin_dir.join(exe_file_name),
                                    )?;
                                }
                            }
                        }
                    }

                    // Little hack here to download core and std lib upon installing `forc`
                    if download_cfg.name == component::FORC {
                        cache_sway_std_libs(self.bin_path.join(component::FORC))?;
                    };
                }
                Err(e) => bail!(
                    "Could not add component {}({}): {}",
                    &download_cfg.name,
                    &download_cfg.version,
                    e
                ),
            }
        } else {
            // We have to iterate here because `fuelup component add forc` has to account for
            // other built-in plugins as well, eg. forc-fmt
            for entry in std::fs::read_dir(
                store.component_dir_path(&download_cfg.name, &download_cfg.version),
            )? {
                let entry = entry?;
                let exe = entry.path();

                if is_executable(exe.as_path()) {
                    if let Some(exe_file_name) = exe.file_name() {
                        hard_or_symlink_file(exe.as_path(), &self.bin_path.join(exe_file_name))?;
                    }
                }
            }
        };

        info!(
            "Installed {} v{} for toolchain '{}'",
            download_cfg.name, download_cfg.version, self.name
        );

        Ok(download_cfg)
    }

    pub fn install_if_nonexistent(&self, description: &DistToolchainDescription) -> Result<()> {
        if !self.exists() {
            info!("toolchain '{}' does not exist; installing", description);
            if let Ok(channel) = Channel::from_dist_channel(description) {
                ensure_dir_exists(&self.bin_path)?;
                let store = Store::from_env()?;
                for cfg in channel.build_download_configs() {
                    if store.has_component(&cfg.name, &cfg.version) {
                        hard_or_symlink_file(
                            &store
                                .component_dir_path(&cfg.name, &cfg.version)
                                .join(&cfg.name),
                            &self.bin_path.join(&cfg.name),
                        )?;
                    } else {
                        let downloaded = store.install_component(&cfg)?;
                        for bin in downloaded {
                            hard_or_symlink_file(&bin, &self.bin_path.join(&cfg.name))?;
                        }
                    }
                }
            }
        };

        Ok(())
    }
    fn remove_executables(&self, component: &str) -> Result<()> {
        let executables = &Components::collect().unwrap().component[component].executables;
        for executable in executables {
            remove_file(self.bin_path.join(executable))
                .with_context(|| format!("failed to remove executable '{executable}'"))?;
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

    #[test]
    fn test_parse_name() -> Result<()> {
        for name in [channel::LATEST, channel::NIGHTLY] {
            let desc = DistToolchainDescription::from_str(name)?;
            assert_eq!(desc.name, DistToolchainName::from_str(name).unwrap());
            assert_eq!(desc.date, None);
            assert_eq!(desc.target, Some(TargetTriple::from_host().unwrap()));
        }

        Ok(())
    }

    #[test]
    fn test_parse_name_should_fail() -> Result<()> {
        let inputs = ["latest-2", "nightly-toolchain"];
        for name in inputs {
            assert!(DistToolchainDescription::from_str(name).is_err());
        }

        Ok(())
    }

    #[test]
    fn test_parse_nightly_date() -> Result<()> {
        let toolchain = format!("{}-{}", channel::NIGHTLY.to_owned(), DATE);
        let desc = DistToolchainDescription::from_str(&toolchain).unwrap();

        assert_eq!(desc.name, DistToolchainName::from_str("nightly").unwrap());
        assert_eq!(desc.date.unwrap().to_string(), DATE);
        assert_eq!(desc.target, None);

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
            let toolchain = format!("{}-{}-{}", channel::NIGHTLY.to_owned(), DATE, target);
            assert!(DistToolchainDescription::from_str(&toolchain).is_err());
            // TODO: Uncomment once target specification is supported
            // see issue #237: https://github.com/FuelLabs/fuelup/issues/237
            // assert_eq!(
            //     desc.name,
            //     DistToolchainName::from_str(channel::NIGHTLY).unwrap()
            // );
            // assert_eq!(desc.date.unwrap().to_string(), DATE);
            // assert_eq!(desc.target.unwrap().to_string(), target);
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
                let desc = DistToolchainDescription::from_str(&toolchain).unwrap();

                assert_eq!(desc.name, DistToolchainName::from_str(name).unwrap());
                assert!(desc.date.is_none());
                assert_eq!(desc.target.unwrap().to_string(), target);
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
        assert!(parse_metadata(DATE_TARGET_APPLE.to_string()).is_err());
        // TODO: Uncomment once target specification is supported
        // see issue #237: https://github.com/FuelLabs/fuelup/issues/237
        //assert_eq!(DATE, date.unwrap().to_string());
        //assert_eq!(TARGET_X86_APPLE, target.unwrap().to_string());
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
