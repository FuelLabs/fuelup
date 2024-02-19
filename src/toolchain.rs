use crate::channel::{self, Channel};
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
use anyhow::{bail, Context, Result};
use component::{self, Components};
use std::collections::VecDeque;
use std::fmt;
use std::fs::{remove_dir_all, remove_file};
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;
use time::Date;
use tracing::{error, info};

pub const RESERVED_TOOLCHAIN_NAMES: &[&str] = &[
    channel::LATEST,
    channel::BETA_1,
    channel::BETA_2,
    channel::BETA_3,
    channel::BETA_4,
    channel::BETA_5,
    channel::NIGHTLY,
    // Stable is reserved, although currently unused.
    channel::STABLE,
];

#[derive(Debug, Eq, PartialEq)]
pub enum DistToolchainName {
    Beta1,
    Beta2,
    Beta3,
    Beta4,
    Beta5,
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
            DistToolchainName::Beta5 => write!(f, "{}", channel::BETA_5),
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
            channel::BETA_5 => Ok(Self::Beta5),
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

impl fmt::Display for DistToolchainDescription {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let target = TargetTriple::from_host().unwrap_or_default();
        match self.date {
            Some(d) => write!(f, "{}-{}-{}", self.name, d, target),
            None => write!(f, "{}-{}", self.name, target),
        }
    }
}

#[inline]
fn consume_back<T>(parts: &mut VecDeque<T>, number: usize) {
    for _ in 0..number {
        parts.pop_back();
    }
}

/// Attempts to parse a date from the front of the parts list, returning the date and consuming the
/// date parts if they are available
fn extract_date(parts: &mut VecDeque<&str>) -> Option<Date> {
    let len = parts.len();
    if len < 3 {
        return None;
    }

    let date_str = format!("{}-{}-{}", parts[len - 3], parts[len - 2], parts[len - 1]);
    match Date::parse(&date_str, DATE_FORMAT) {
        Ok(d) => {
            consume_back(parts, 3);
            Some(d)
        }
        Err(_) => None,
    }
}

/// Attemps to parse the target from a vector of parts, returning the target and consuming the
/// target parts if they are available
fn extract_target(parts: &mut VecDeque<&str>) -> Option<TargetTriple> {
    if parts.len() < 3 {
        return None;
    }

    let len = parts.len();
    let target_str = format!("{}-{}-{}", parts[len - 3], parts[len - 2], parts[len - 1]);
    match TargetTriple::new(&target_str) {
        Ok(t) => {
            consume_back(parts, 3);
            Some(t)
        }
        Err(_) => {
            if parts.len() < 4 {
                return None;
            }

            let target_str = format!(
                "{}-{}-{}-{}",
                parts[len - 4],
                parts[len - 3],
                parts[len - 2],
                parts[len - 1]
            );
            match TargetTriple::new(&target_str) {
                Ok(t) => {
                    consume_back(parts, 4);
                    Some(t)
                }
                Err(_) => None,
            }
        }
    }
}

/// Parses a distributable toolchain description from a string.
///
/// The supported formats are:
///     <channel>
///     <channel>-<target>
///     <channel>-<YYYY-MM-DD>
///     <channel>-<YYYY-MM-DD>-<target>
///     <channel>-<target>-<YYYY-MM-DD>
/// The parsing begings from the end of the string, so the target is the last part of the string,
/// then the date and finally the name
impl FromStr for DistToolchainDescription {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        if s.ends_with('-') && s.matches('-').count() == 1 {
            bail!("Invalid distributable toolchain name '{}'", s);
        }

        let mut parts = s.split('-').collect::<VecDeque<_>>();
        match parts.len() {
            1 => Ok(Self {
                name: DistToolchainName::from_str(parts[0])?,
                target: TargetTriple::from_host().ok(),
                date: None,
            }),
            _ => {
                let date = extract_date(&mut parts);
                let target = extract_target(&mut parts);
                let date = if date.is_none() && target.is_some() {
                    // if date is not present but target is, then the date is the last part of the
                    // name could be date, so we try to parse it
                    extract_date(&mut parts)
                } else {
                    date
                };

                let name = parts.into_iter().collect::<Vec<_>>().join("-");
                Ok(Self {
                    name: DistToolchainName::from_str(&name)?,
                    date,
                    target: if target.is_none() {
                        TargetTriple::from_host().ok()
                    } else {
                        target
                    },
                })
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
            match self_update(true) {
                Ok(()) => info!("fuelup installed."),
                Err(e) => bail!("Could not install fuelup: {}", e),
            };
        }

        let store = Store::from_env()?;

        if !store.has_component(&download_cfg.name, &download_cfg.version)
            || !self.has_component(&download_cfg.name)
        {
            info!(
                "\nAdding component {} v{} to '{}'",
                &download_cfg.name, &download_cfg.version, self.name
            );

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

            info!(
                "Installed {} v{} for toolchain '{}'",
                download_cfg.name, download_cfg.version, self.name
            );
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
    use crate::channel::{CHANNELS, STABLE};

    use super::*;

    const DATE: &str = "2022-08-29";
    const INVALID_DATES: [&str; 5] = [
        "2022-08-2",
        "2022-08-299",
        "22-08-29",
        "2022-08",
        "2022-08-",
    ];
    const INVALID_CHANNELS: [&str; 4] = ["latest-", "latest-2", "nightly-toolchain", STABLE];
    const TARGET_X86_APPLE: &str = "x86_64-apple-darwin";
    const TARGET_ARM_APPLE: &str = "aarch64-apple-darwin";
    const TARGET_X86_LINUX: &str = "x86_64-unknown-linux-gnu";
    const TARGET_ARM_LINUX: &str = "aarch64-unknown-linux-gnu";
    const TARGETS: [&str; 4] = [
        TARGET_ARM_APPLE,
        TARGET_ARM_LINUX,
        TARGET_X86_APPLE,
        TARGET_X86_LINUX,
    ];

    #[test]
    fn test_parse_description_from_channel() {
        for channel in CHANNELS {
            let desc = DistToolchainDescription::from_str(channel).expect("desc");
            assert_eq!(desc.name, DistToolchainName::from_str(channel).unwrap());
            assert_eq!(desc.date, None);
            assert_eq!(desc.target, Some(TargetTriple::from_host().unwrap()));
        }
    }

    #[test]
    fn test_parse_description_from_target_error() {
        for target in TARGETS {
            DistToolchainDescription::from_str(target).expect_err("target");
        }
    }

    #[test]
    fn test_parse_description_from_date_error() {
        DistToolchainDescription::from_str(DATE).expect_err("date");

        for date in INVALID_DATES {
            DistToolchainDescription::from_str(date).expect_err("invalid date");
        }
    }

    #[test]
    fn test_parse_description_channel_target() {
        for channel in CHANNELS {
            for target in TARGETS {
                let desc_string = format!("{}-{}", channel, target);
                let desc = DistToolchainDescription::from_str(&desc_string).expect("desc");
                assert_eq!(desc.name, DistToolchainName::from_str(channel).unwrap());
                assert_eq!(desc.date, None);
                assert_eq!(desc.target.unwrap().to_string(), target.to_string());
            }
        }
    }

    #[test]
    fn test_parse_description_channel_date() {
        let target = TargetTriple::from_host().unwrap();
        for channel in CHANNELS {
            let desc_string = format!("{}-{}", channel, DATE);
            let desc = DistToolchainDescription::from_str(&desc_string).expect("desc");
            assert_eq!(desc.name, DistToolchainName::from_str(channel).unwrap());
            assert_eq!(desc.date.unwrap().to_string(), DATE);
            assert_eq!(desc.target.unwrap(), target);
        }
    }

    #[test]
    fn test_parse_description_channel_invalid_date_error() {
        for channel in CHANNELS {
            for date in INVALID_DATES {
                let desc_string = format!("{}-{}", channel, date);
                DistToolchainDescription::from_str(&desc_string).expect_err("invalid date");
            }
        }
    }

    #[test]
    fn test_parse_description_date_channel_error() {
        for channel in CHANNELS {
            let desc_string = format!("{}-{}", DATE, channel);
            DistToolchainDescription::from_str(&desc_string).expect_err("date channel");
        }
    }

    #[test]
    fn test_parse_description_date_target_error() {
        for target in TARGETS {
            let desc_string = format!("{}-{}", DATE, target);
            DistToolchainDescription::from_str(&desc_string).expect_err("date target");
        }
    }

    #[test]
    fn test_parse_description_target_channel_error() {
        for target in TARGETS {
            for channel in CHANNELS {
                let desc_string = format!("{}-{}", target, channel);
                DistToolchainDescription::from_str(&desc_string).expect_err("target channel");
            }
        }
    }

    #[test]
    fn test_parse_description_target_date_error() {
        for target in TARGETS {
            let desc_string = format!("{}-{}", target, DATE);
            DistToolchainDescription::from_str(&desc_string).expect_err("target date");
        }
    }

    #[test]
    fn test_parse_description_channel_target_date() {
        for channel in CHANNELS {
            for target in TARGETS {
                let desc_string = format!("{}-{}-{}", channel, target, DATE);
                let desc = DistToolchainDescription::from_str(&desc_string).expect("desc");
                assert_eq!(desc.name, DistToolchainName::from_str(channel).unwrap());
                assert_eq!(desc.date.unwrap().to_string(), DATE);
                assert_eq!(desc.target.unwrap().to_string(), target.to_string());
            }
        }
    }

    #[test]
    fn test_parse_description_channel_date_target() {
        for channel in CHANNELS {
            for target in TARGETS {
                let desc_string = format!("{}-{}-{}", channel, DATE, target);
                let desc =
                    DistToolchainDescription::from_str(&desc_string).expect("channel date target");
                assert_eq!(desc.name, DistToolchainName::from_str(channel).unwrap());
                assert_eq!(desc.date.unwrap().to_string(), DATE);
                assert_eq!(desc.target.unwrap().to_string(), target.to_string());
            }
        }
    }

    #[test]
    fn test_parse_description_channel_target_date_error() {
        for channel in CHANNELS {
            for target in TARGETS {
                let desc_string = format!("{}-{}-{}", target, channel, DATE);
                DistToolchainDescription::from_str(&desc_string).expect_err("target channel date");

                let desc_string = format!("{}-{}-{}", target, DATE, channel);
                DistToolchainDescription::from_str(&desc_string).expect_err("target date channel");

                let desc_string = format!("{}-{}-{}", DATE, channel, target);
                DistToolchainDescription::from_str(&desc_string).expect_err("date channel target");

                let desc_string = format!("{}-{}-{}", DATE, target, channel);
                DistToolchainDescription::from_str(&desc_string).expect_err("date target channel");
            }
        }
    }

    #[test]
    fn test_parse_description_invalid_channel_error() {
        for channel in INVALID_CHANNELS {
            DistToolchainDescription::from_str(channel).expect_err("invalid channel");
        }
    }
}
