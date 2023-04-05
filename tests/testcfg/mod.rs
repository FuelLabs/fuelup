use anyhow::Result;
use fuelup::channel::{BETA_1, LATEST, NIGHTLY};
use fuelup::constants::FUEL_TOOLCHAIN_TOML_FILE;
use fuelup::file::hard_or_symlink_file;
use fuelup::settings::SettingsFile;
use fuelup::target_triple::TargetTriple;
use fuelup::toolchain_override::{self, OverrideCfg, ToolchainCfg, ToolchainOverride};
use semver::Version;
use std::os::unix::fs::OpenOptionsExt;
use std::str::FromStr;
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
};
use tempfile::tempdir;

/// State of the virtual environment for which tests are ran against.
pub enum FuelupState {
    /// Inits a state where `latest`, `nightly`, `nightly-2022-08-30` toolchains are installed.
    AllInstalled,
    /// Inits an empty state with no toolchains.
    Empty,
    /// Inits a state with only the `latest` toolchain.
    LatestToolchainInstalled,
    /// Inits a state with only the `nightly` toolchain.
    NightlyInstalled,
    /// Inits a state with only the `nightly-2022-08-30` toolchain.
    NightlyDateInstalled,
    /// Inits a state with the `latest` and custom `my-toolchain` toolchains.
    LatestAndCustomInstalled,
    /// Inits a state with the `latest` toolchain installed, with conflicting binaries in
    /// other well-known directories like `.cargo` or `.local`.
    FuelupUpdateConflict,
    /// Inits a state with the `nightly` and `nightly-2022-08-30` toolchains.
    NightlyAndNightlyDateInstalled,
    /// Inits a state with only the `beta-1` toolchain.
    Beta1Installed,
    /// Inits a state with the `latest` toolchain, with `beta-1` declared within fuel-toolchain.toml.
    LatestWithBetaOverride,
}

#[derive(Debug)]
pub struct TestCfg {
    /// The path to the test environment's fuelup executable. This should usually be <TMP_DIR>/.fuelup/bin/fuelup.
    /// This should be used to execute fuelup in the test environment.
    pub fuelup_path: PathBuf,
    /// The path to the test environment's fuelup/bin directory. This should usually be <TMP_DIR>/.fuelup/bin/.
    /// This should be used to execute other binaries (eg. forc) in the test environment.
    pub fuelup_bin_dirpath: PathBuf,
    /// The path to the test environment's home. This should usually be a created tempfile::tempdir::TempDir.
    pub home: PathBuf,
}

#[derive(Debug)]
pub struct TestOutput {
    pub stdout: String,
    pub stderr: String,
    pub status: ExitStatus,
}

pub const DATE: &str = "2022-08-30";
pub const CUSTOM_TOOLCHAIN_NAME: &str = "my-toolchain";

const VERSION: &Version = &Version::new(0, 1, 0);
const VERSION_2: &Version = &Version::new(0, 2, 0);

pub static ALL_BINS: &[&str] = &[
    "forc",
    "forc-deploy",
    "forc-doc",
    "forc-explore",
    "forc-fmt",
    "forc-index",
    "forc-lsp",
    "forc-run",
    "forc-tx",
    "forc-wallet",
    "fuel-core",
    "fuel-indexer",
];

impl TestCfg {
    pub fn new(fuelup_path: PathBuf, fuelup_bin_dirpath: PathBuf, home: PathBuf) -> Self {
        Self {
            fuelup_path,
            fuelup_bin_dirpath,
            home,
        }
    }

    pub fn toolchains_dir(&self) -> PathBuf {
        self.home.join(".fuelup").join("toolchains")
    }

    pub fn toolchain_bin_dir(&self, toolchain: &str) -> PathBuf {
        self.home
            .join(".fuelup")
            .join("toolchains")
            .join(toolchain)
            .join("bin")
    }

    pub fn settings_file(&self) -> SettingsFile {
        SettingsFile::new(self.home.join(".fuelup").join("settings.toml"))
    }

    pub fn default_toolchain(&self) -> Option<String> {
        self.settings_file()
            .with(|s| Ok(s.default_toolchain.clone()))
            .unwrap()
    }

    pub fn exec(&mut self, proc_name: &str, args: &[&str]) -> TestOutput {
        let output = Command::new(proc_name)
            .args(args)
            .current_dir(&self.home)
            .env("HOME", &self.home)
            .env(
                "PATH",
                format!("{}", &self.home.join(".fuelup/bin").display(),),
            )
            .env("TERM", "dumb")
            .output()
            .expect("Failed to execute command");
        let stdout = String::from_utf8(output.stdout).unwrap();
        let stderr = String::from_utf8(output.stderr).unwrap();
        TestOutput {
            stdout,
            stderr,
            status: output.status,
        }
    }

    pub fn forc(&mut self, args: &[&str]) -> TestOutput {
        self.exec("forc", args)
    }

    pub fn fuelup(&mut self, args: &[&str]) -> TestOutput {
        let output = Command::new(&self.fuelup_path)
            .args(args)
            .current_dir(&self.home)
            .env("HOME", &self.home)
            .env("CARGO_HOME", self.home.join(".cargo").to_str().unwrap())
            .env(
                "PATH",
                format!(
                    "{}:{}",
                    &self.home.join(".local/bin").display(),
                    &self.home.join(".cargo/bin").display()
                ),
            )
            .env("TERM", "dumb")
            .output()
            .expect("Failed to execute command");
        let stdout = String::from_utf8(output.stdout).unwrap();
        let stderr = String::from_utf8(output.stderr).unwrap();
        TestOutput {
            stdout,
            stderr,
            status: output.status,
        }
    }
}

#[cfg(unix)]
fn create_fuel_executable(exe_name: &str, path: &Path, version: &Version) -> std::io::Result<()> {
    use std::io::Write;

    let mut exe = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .mode(0o770)
        .open(path)?;
    exe.write_all(&format!("#!/bin/sh\n\necho {exe_name} {version}").into_bytes())?;

    Ok(())
}

#[cfg(windows)]
fn create_fuel_executable(path: &Path) -> std::io::Result<()> {
    fs::File::create(path)?;
    Ok(())
}

fn setup_toolchain(fuelup_home_path: &Path, toolchain: &str) -> Result<()> {
    let bin_dir = fuelup_home_path
        .join("toolchains")
        .join(toolchain)
        .join("bin");
    fs::create_dir_all(&bin_dir).expect("Failed to create temporary latest toolchain bin dir");

    for bin in ALL_BINS {
        let version = match toolchain.starts_with("latest") {
            true => VERSION,
            _ => VERSION_2,
        };
        create_fuel_executable(bin, &bin_dir.join(bin), version)?;
    }

    Ok(())
}

fn setup_settings_file(settings_dir: &Path, default_toolchain: &str) -> Result<()> {
    let settings_path = settings_dir.join("settings.toml");
    fs::write(
        settings_path,
        format!("default_toolchain = \"{default_toolchain}\""),
    )
    .expect("Failed to copy settings");
    Ok(())
}

pub fn setup_override_file(toolchain_override: ToolchainOverride) -> Result<()> {
    let document = toolchain_override.to_toml();

    fs::write(toolchain_override.path, document.to_string())
        .unwrap_or_else(|_| panic!("Failed to write {FUEL_TOOLCHAIN_TOML_FILE}"));

    Ok(())
}

/// Based on a given FuelupState, sets up a temporary directory with all the necessary mock
/// files and directories and provides a TestCfg to test fuelup.
pub fn setup(state: FuelupState, f: &dyn Fn(&mut TestCfg)) -> Result<()> {
    let testdir = tempdir().unwrap();
    let tmp_home = testdir.path();

    let tmp_fuelup_root_path = tmp_home.join(".fuelup");
    let tmp_fuelup_bin_dir_path = tmp_home.join(".fuelup").join("bin");

    fs::create_dir_all(&tmp_fuelup_bin_dir_path).unwrap();
    fs::create_dir(tmp_fuelup_root_path.join("toolchains")).unwrap();

    let root = env::current_exe()
        .unwrap()
        .parent()
        .expect("fuelup's directory")
        .to_path_buf();
    hard_or_symlink_file(
        &root.parent().unwrap().join("fuelup"),
        &tmp_fuelup_bin_dir_path.join("fuelup"),
    )?;

    for bin in ALL_BINS {
        hard_or_symlink_file(
            &tmp_fuelup_bin_dir_path.join("fuelup"),
            &tmp_fuelup_bin_dir_path.join(bin),
        )
        .unwrap()
    }

    let target = TargetTriple::from_host().unwrap();
    let latest = format!("{LATEST}-{target}");
    let nightly = format!("{NIGHTLY}-{target}");
    let beta_1 = format!("{BETA_1}-{target}");

    match state {
        FuelupState::Empty => {}
        FuelupState::AllInstalled => {
            setup_toolchain(&tmp_fuelup_root_path, &latest)?;
            setup_toolchain(&tmp_fuelup_root_path, &nightly)?;
            setup_toolchain(&tmp_fuelup_root_path, &format!("nightly-{DATE}-{target}"))?;
            setup_settings_file(&tmp_fuelup_root_path, &latest)?;
        }
        FuelupState::LatestToolchainInstalled => {
            setup_toolchain(&tmp_fuelup_root_path, &latest)?;
            setup_settings_file(&tmp_fuelup_root_path, &latest)?;
        }
        FuelupState::FuelupUpdateConflict => {
            setup_toolchain(&tmp_fuelup_root_path, &latest)?;
            setup_settings_file(&tmp_fuelup_root_path, &latest)?;

            fs::create_dir_all(tmp_home.join(".local/bin"))?;
            create_fuel_executable("forc", &tmp_home.join(".local/bin/forc"), VERSION)?;

            create_fuel_executable(
                "forc-wallet",
                &tmp_home.join(".local/bin/forc-wallet"),
                VERSION,
            )?;
            create_fuel_executable(
                "forc-wallet",
                &tmp_home.join(".fuelup/bin/forc-wallet"),
                VERSION,
            )?;

            fs::create_dir_all(tmp_home.join(".cargo/bin"))?;

            create_fuel_executable(
                "forc-explore",
                &tmp_home.join(".cargo/bin/forc-explore"),
                VERSION,
            )?;
            create_fuel_executable(
                "forc-explore",
                &tmp_home.join(".fuelup/bin/forc-explore"),
                VERSION,
            )?;

            create_fuel_executable("fuel-core", &tmp_home.join(".cargo/bin/fuel-core"), VERSION)?;
        }
        FuelupState::NightlyInstalled => {
            setup_toolchain(&tmp_fuelup_root_path, &nightly)?;
            setup_settings_file(&tmp_fuelup_root_path, &nightly)?;
        }
        FuelupState::NightlyDateInstalled => {
            setup_toolchain(&tmp_fuelup_root_path, &format!("nightly-{DATE}-{target}"))?;
            setup_settings_file(&tmp_fuelup_root_path, &format!("nightly-{DATE}-{target}"))?;
        }
        FuelupState::LatestAndCustomInstalled => {
            setup_toolchain(&tmp_fuelup_root_path, &latest)?;
            setup_toolchain(&tmp_fuelup_root_path, CUSTOM_TOOLCHAIN_NAME)?;
            setup_settings_file(&tmp_fuelup_root_path, &latest)?;
        }
        FuelupState::NightlyAndNightlyDateInstalled => {
            setup_toolchain(&tmp_fuelup_root_path, &nightly)?;
            setup_toolchain(&tmp_fuelup_root_path, &format!("nightly-{DATE}-{target}"))?;
            setup_settings_file(&tmp_fuelup_root_path, &nightly)?;
        }
        FuelupState::Beta1Installed => {
            setup_toolchain(&tmp_fuelup_root_path, &beta_1)?;
            setup_toolchain(&tmp_fuelup_root_path, &format!("beta-1-{DATE}-{target}"))?;
            setup_settings_file(&tmp_fuelup_root_path, &beta_1)?;
        }
        FuelupState::LatestWithBetaOverride => {
            setup_toolchain(&tmp_fuelup_root_path, &latest)?;
            setup_settings_file(&tmp_fuelup_root_path, &latest)?;
            setup_override_file(ToolchainOverride {
                cfg: OverrideCfg::new(
                    ToolchainCfg {
                        channel: toolchain_override::Channel::from_str("beta-1").unwrap(),
                    },
                    None,
                ),
                path: tmp_home.join(FUEL_TOOLCHAIN_TOML_FILE),
            })?;
        }
    }

    f(&mut TestCfg::new(
        tmp_fuelup_bin_dir_path.join("fuelup"),
        tmp_fuelup_bin_dir_path,
        tmp_home.to_path_buf(),
    ));

    Ok(())
}
