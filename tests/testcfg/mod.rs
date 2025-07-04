use anyhow::Result;
use component::Component;
use fuelup::channel::{LATEST, NIGHTLY, TESTNET};
use fuelup::constants::FUEL_TOOLCHAIN_TOML_FILE;
use fuelup::file::hard_or_symlink_file;
use fuelup::settings::SettingsFile;
use fuelup::target_triple::TargetTriple;
use fuelup::toolchain::Toolchain;
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
    /// Inits a state with the `latest` toolchain, with `testnet` declared within
    /// fuel-toolchain.toml.
    LatestWithTestnetOverride,
}

#[derive(Debug)]
pub struct TestCfg {
    /// The path to the test environment's fuelup executable. This should usually be
    /// <TMP_DIR>/.fuelup/bin/fuelup. This should be used to execute fuelup in the test
    /// environment.
    pub fuelup_path: PathBuf,
    /// The path to the test environment's fuelup/bin directory. This should usually be
    /// <TMP_DIR>/.fuelup/bin/. This should be used to execute other binaries (eg. forc) in the
    /// test environment.
    pub fuelup_bin_dirpath: PathBuf,
    /// The path to the test environment's home. This should usually be a created
    /// tempfile::tempdir::TempDir.
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
    "forc-call",
    "forc-crypto",
    "forc-debug",
    "forc-deploy",
    "forc-doc",
    "forc-fmt",
    "forc-index",
    "forc-lsp",
    "forc-migrate",
    "forc-node",
    "forc-publish",
    "forc-run",
    "forc-submit",
    "forc-tx",
    "forc-wallet",
    "fuel-core",
    "fuel-core-keygen",
    "fuel-indexer",
];

/// Returns a `String` containing yesterday's UTC date in ISO-8601 format
///
/// # Examples
///
/// ```rust
/// use testcfg::yesterday;
/// use regex::Regex;
///
/// let yesterday = yesterday();
/// let re = Regex::new(r"20\d{2}-\d{2}-\d{2}").unwrap();
///
/// assert!(re.is_match(&yesterday));
/// ```
pub fn yesterday() -> String {
    // TODO: once https://github.com/FuelLabs/fuelup/issues/739 is closed, this
    // can be reverted back to being dynamically calculated as actual yesterday
    //
    // let current_date = Utc::now();
    // let yesterday = current_date - Duration::days(1);
    // yesterday.format("%Y-%m-%d").to_string()
    "2025-05-26".to_string()
}

impl TestCfg {
    pub fn new(fuelup_path: PathBuf, fuelup_bin_dirpath: PathBuf, home: PathBuf) -> Self {
        Self {
            fuelup_path,
            fuelup_bin_dirpath,
            home,
        }
    }

    pub fn fuelup_dir(&self) -> PathBuf {
        self.home.join(".fuelup")
    }

    pub fn toolchains_dir(&self) -> PathBuf {
        self.fuelup_dir().join("toolchains")
    }

    pub fn toolchain_bin_dir(&self, toolchain: &str) -> PathBuf {
        self.fuelup_dir()
            .join("toolchains")
            .join(toolchain)
            .join("bin")
    }

    pub fn settings_file(&self) -> SettingsFile {
        SettingsFile::new(self.fuelup_dir().join("settings.toml"))
    }

    pub fn default_toolchain(&self) -> Option<String> {
        self.settings_file()
            .with(|s| Ok(s.default_toolchain.clone()))
            .unwrap()
    }

    /// A function for executing binaries within the fuelup test configuration.
    ///
    /// This invokes std::process::Command with some default environment variables
    /// set up nicely for testing fuelup and its managed binaries.
    pub fn exec(&mut self, proc_name: &str, args: &[&str]) -> TestOutput {
        let path = self.fuelup_bin_dirpath.join(proc_name);
        let output = Command::new(path)
            .args(args)
            .current_dir(&self.home)
            .env("HOME", &self.home)
            .env("CARGO_HOME", self.home.join(".cargo"))
            .env(
                "PATH",
                format!(
                    "{}:{}:{}",
                    &self.home.join(".local/bin").display(),
                    &self.home.join(".cargo/bin").display(),
                    &self.home.join(".fuelup/bin").display(),
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

    /// A convenience wrapper for executing 'forc' binaries within the fuelup test configuration.
    /// This is just a testcfg::exec() call with "forc" as its first argument.
    pub fn forc(&mut self, args: &[&str]) -> TestOutput {
        self.exec("forc", args)
    }

    /// A convenience wrapper for executing 'fuelup' within the fuelup test configuration.
    /// This is just a testcfg::exec() call with "fuelup" as its first argument.
    pub fn fuelup(&mut self, args: &[&str]) -> TestOutput {
        self.exec("fuelup", args)
    }
}

#[cfg(unix)]
fn create_fuel_executable(exe_name: &str, path: &Path, version: &Version) -> std::io::Result<()> {
    use std::io::Write;

    let mut exe = fs::OpenOptions::new()
        .create(true)
        .truncate(true)
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

/// Deletes the default toolchain override `Toolchain` from the test environment
///
/// # Arguments:
///
/// * `cfg` - The `TestCfg` test environment
///
/// # Examples
///
/// ```no_run
/// use testcfg::{self, delete_default_toolchain_override_toolchain, FuelupState};
///
/// testcfg::setup(FuelupState::LatestToolchainInstalled, &|cfg| {
///     delete_default_toolchain_override_toolchain(cfg);
/// });
/// ```
pub fn delete_default_toolchain_override_toolchain(cfg: &TestCfg) {
    let toolchain = get_default_toolchain_override_toolchain();
    delete_toolchain(cfg, &toolchain);
}

/// Deletes the `Toolchain` from the test environment
///
/// # Arguments:
///
/// * `cfg` - The `TestCfg` test environment
///
/// * `toolchain` - The `Toolchain` to be deleted
///
/// # Examples
///
/// ```no_run
/// use testcfg::{self, delete_toolchain, yesterday, FuelupState};
/// use fuelup::toolchain::Toolchain;
///
/// testcfg::setup(FuelupState::LatestToolchainInstalled, &|cfg| {
///     let toolchain = Toolchain::new(&format!("nightly-{}", yesterday())).unwrap();
///     delete_toolchain(&cfg, &toolchain);
/// });
/// ```
pub fn delete_toolchain(cfg: &TestCfg, toolchain: &Toolchain) {
    let toolchain_bin_dir = cfg.toolchain_bin_dir(toolchain.name.as_str());
    let toolchain_dir = &toolchain_bin_dir.parent().unwrap();
    std::fs::remove_dir_all(toolchain_dir).unwrap();
}

/// Returns the default toolchain override `Toolchain` for the test environment
///
/// # Examples
///
/// ```no_run
/// use testcfg::{self, get_default_toolchain_override_toolchain, FuelupState};
///
/// testcfg::setup(FuelupState::LatestToolchainInstalled, &|cfg| {
///     let toolchain = get_default_toolchain_override_toolchain();
///     assert_eq!(toolchain.name, format!("nightly-{}", yesterday()));
/// });
/// ```
pub fn get_default_toolchain_override_toolchain() -> Toolchain {
    Toolchain::new(format!("nightly-{}", yesterday()).as_str()).unwrap()
}

fn setup_toolchain(fuelup_home_path: &Path, toolchain: &str) -> Result<()> {
    let bin_dir = fuelup_home_path
        .join("toolchains")
        .join(toolchain)
        .join("bin");
    fs::create_dir_all(&bin_dir).expect("Failed to create temporary latest toolchain bin dir");
    let fuelup_bin_dirpath = fuelup_home_path.join("bin");

    for bin in ALL_BINS {
        let version = match toolchain.starts_with("latest") {
            true => VERSION,
            _ => VERSION_2,
        };
        create_fuel_executable(bin, &bin_dir.join(bin), version)?;

        if !fuelup_bin_dirpath.join(bin).exists() {
            hard_or_symlink_file(
                &fuelup_bin_dirpath.join("fuelup"),
                &fuelup_bin_dirpath.join(bin),
            )?;
        }
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

/// Creates the default toolchain override for the test environment
///
/// # Arguments:
///
/// * `cfg` - The `TestCfg` test environment
///
/// * `component_name` - If supplied, it will be used as a `Component` override
///   in the "components" section of the toolchain override file. Otherwise, the
///   "components" section will be empty.
///
/// # Examples
///
/// ```no_run
/// use testcfg::{self, setup_default_override_file, FuelupState};
///
/// testcfg::setup(FuelupState::LatestToolchainInstalled, &|cfg| {
///     setup_default_override_file(cfg, Some("forc-fmt"));
/// });
/// ```
pub fn setup_default_override_file(cfg: &TestCfg, component_name: Option<&str>) {
    // TODO: "0.61.0" is a placeholder until #666 is merged. Then we can use
    // Component::resolve_from_name() to get a valid version (i.e the latest)
    // via download::get_latest_version() as the component override version

    let toolchain_override = ToolchainOverride {
        cfg: OverrideCfg::new(
            ToolchainCfg {
                channel: toolchain_override::Channel::from_str(&format!("nightly-{}", yesterday()))
                    .unwrap(),
            },
            component_name.map(|c| {
                vec![(c.to_string(), "0.61.0".parse().unwrap())]
                    .into_iter()
                    .collect()
            }),
        ),
        path: cfg.home.join(FUEL_TOOLCHAIN_TOML_FILE),
    };

    setup_override_file(toolchain_override.clone()).unwrap()
}

/// Based on a given FuelupState, sets up a temporary directory with all the necessary mock
/// files and directories and provides a TestCfg to test fuelup.
pub fn setup(state: FuelupState, f: &dyn Fn(&mut TestCfg)) -> Result<()> {
    let testdir = tempdir().unwrap();
    let tmp_home = testdir.path().canonicalize()?;

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

    let target = TargetTriple::from_host().unwrap();
    let latest = format!("{LATEST}-{target}");
    let nightly = format!("{NIGHTLY}-{target}");
    let testnet = format!("{TESTNET}-{target}");

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

            // Intentionally create 'binaries' that conflict with update
            fs::create_dir_all(tmp_home.join(".local/bin"))?;
            create_fuel_executable("forc", &tmp_home.join(".local/bin/forc"), VERSION)?;

            create_fuel_executable(
                "forc-wallet",
                &tmp_home.join(".local/bin/forc-wallet"),
                VERSION,
            )?;
            fs::create_dir_all(tmp_home.join(".cargo/bin"))?;

            create_fuel_executable("fuel-core", &tmp_home.join(".cargo/bin/fuel-core"), VERSION)?;

            // Here we intentionally remove some of the 'binaries' that were linked in the
            // setup_toolchain() step so we can expect some error messages from tests.
            fs::remove_file(tmp_home.join(".fuelup/bin/forc"))?;
            fs::remove_file(tmp_home.join(".fuelup/bin/fuel-core"))?;
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
        FuelupState::LatestWithTestnetOverride => {
            setup_toolchain(&tmp_fuelup_root_path, &latest)?;
            setup_settings_file(&tmp_fuelup_root_path, &latest)?;
            setup_override_file(ToolchainOverride {
                cfg: OverrideCfg::new(
                    ToolchainCfg {
                        channel: toolchain_override::Channel::from_str(&testnet).unwrap(),
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

/// Verifies all `Component` executables exist in the default toolchain override
/// `Toolchain`'s bin directory
///
/// # Arguments:
///
/// * `cfg` - The `TestCfg` test environment
///
/// * `component` - The `Component` to check executables from
///
/// # Examples
///
/// ```no_run
/// use testcfg::{self, verify_default_toolchain_override_toolchain_executables, FuelupState};
/// use fuelup::component::Component;
///
/// testcfg::setup(FuelupState::LatestToolchainInstalled, &|cfg| {
///     let component = Component::from_name("forc-fmt").unwrap();
///     verify_default_toolchain_override_toolchain_executables(cfg, Some(&component));
/// });
/// ```
pub fn verify_default_toolchain_override_toolchain_executables(
    cfg: &TestCfg,
    component: Option<&Component>,
) {
    let toolchain = get_default_toolchain_override_toolchain();
    verify_toolchain_executables(cfg, component, &toolchain);
}

/// Verifies all `Component` executables exist in the `Toolchain`'s bin directory
///
/// # Arguments:
///
/// * `cfg` - The `TestCfg` test environment
///
/// * `component` - The `Component` to check executables from
///
/// * `toolchain` - The `Toolchain` to check executables files exist in
///
/// # Examples
///
/// ```no_run
/// use testcfg::{self, verify_toolchain_executables, yesterday, FuelupState};
/// use fuelup::component::Component;
/// use fuelup::toolchain::Toolchain;
///
/// testcfg::setup(FuelupState::LatestToolchainInstalled, &|cfg| {
///     let component = Component::from_name("forc-fmt").unwrap();
///     let toolchain = Toolchain::new(&format!("nightly-{}", yesterday())).unwrap();
///     verify_toolchain_executables(cfg, Some(&component), &toolchain);
/// });
/// ```
pub fn verify_toolchain_executables(
    cfg: &TestCfg,
    component: Option<&Component>,
    toolchain: &Toolchain,
) {
    let toolchain_bin_dir = cfg.toolchain_bin_dir(toolchain.name.as_str());
    let executables = component
        .map(|c| c.executables.clone())
        .unwrap_or(vec!["forc".to_string()]);

    for executable in executables {
        assert!(
            toolchain_bin_dir.join(&executable).exists(),
            "Executable '{}' not found in '{}'",
            executable,
            toolchain_bin_dir.to_string_lossy(),
        );
    }
}
