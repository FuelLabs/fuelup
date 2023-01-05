use anyhow::Result;
use fuelup::constants::FUEL_TOOLCHAIN_TOML_FILE;
use fuelup::settings::SettingsFile;
use fuelup::target_triple::TargetTriple;
use fuelup::toolchain_override::{OverrideCfg, ToolchainCfg, ToolchainOverride};
use std::os::unix::fs::OpenOptionsExt;
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
};
use tempfile::tempdir;

pub enum FuelupState {
    AllInstalled,
    Empty,
    LatestToolchainInstalled,
    FuelupUpdateConflict,
    NightlyInstalled,
    NightlyDateInstalled,
    LatestAndCustomInstalled,
    LatestAndNightlyInstalled,
    NightlyAndNightlyDateInstalled,
    Beta1Installed,
    LatestAndNightlyWithBetaOverride,
    LatestAndCustomWithCustomOverride,
}

#[derive(Debug)]
pub struct TestCfg {
    pub fuelup_path: PathBuf,
    pub root: PathBuf,
    pub home: PathBuf,
}

#[derive(Debug)]
pub struct TestOutput {
    pub stdout: String,
    pub stderr: String,
    pub status: ExitStatus,
}

pub const DATE: &str = "2022-08-30";

pub static ALL_BINS: &[&str] = &[
    "forc",
    "forc-deploy",
    "forc-explore",
    "forc-fmt",
    "forc-lsp",
    "forc-run",
    "forc-wallet",
    "fuel-core",
];

impl TestCfg {
    pub fn new(fuelup_path: PathBuf, root: PathBuf, home: PathBuf) -> Self {
        Self {
            fuelup_path,
            root,
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
fn create_fuel_executable(path: &Path) -> std::io::Result<()> {
    fs::OpenOptions::new()
        .create(true)
        .write(true)
        .mode(0o770)
        .open(path)?;
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
        create_fuel_executable(&bin_dir.join(bin))?;
    }

    Ok(())
}

fn setup_settings_file(settings_dir: &Path, default_toolchain: &str) -> Result<()> {
    let settings_path = settings_dir.join("settings.toml");
    fs::write(
        settings_path,
        format!("default_toolchain = \"{}\"", default_toolchain),
    )
    .expect("Failed to copy settings");
    Ok(())
}

fn setup_override_file(toolchain_override: ToolchainOverride) -> Result<()> {
    let document = toolchain_override.to_string()?;

    fs::write(toolchain_override.path, document)
        .unwrap_or_else(|_| panic!("Failed to write {}", FUEL_TOOLCHAIN_TOML_FILE));
    Ok(())
}

pub fn setup(state: FuelupState, f: &dyn Fn(&mut TestCfg)) -> Result<()> {
    let root = env::current_exe()
        .unwrap()
        .parent()
        .expect("fuelup's directory")
        .to_path_buf();

    let testdir = tempdir().unwrap();
    let tmp_home = testdir.path();

    let tmp_fuelup_root_path = tmp_home.join(".fuelup");
    let tmp_fuelup_bin_dir_path = tmp_home.join(".fuelup").join("bin");
    fs::create_dir(&tmp_fuelup_root_path).unwrap();
    fs::create_dir(&tmp_fuelup_bin_dir_path).unwrap();
    fs::create_dir(tmp_fuelup_root_path.join("toolchains")).unwrap();
    fs::hard_link(
        root.parent().unwrap().join("fuelup"),
        tmp_fuelup_bin_dir_path.join("fuelup"),
    )?;

    let target = TargetTriple::from_host().unwrap();
    let latest = format!("latest-{}", target);
    let nightly = format!("nightly-{}", target);
    let beta_1 = format!("beta-1-{}", target);

    match state {
        FuelupState::Empty => {}
        FuelupState::AllInstalled => {
            setup_toolchain(&tmp_fuelup_root_path, &latest)?;
            setup_toolchain(&tmp_fuelup_root_path, &nightly)?;
            setup_toolchain(
                &tmp_fuelup_root_path,
                &format!("nightly-{}-{}", DATE, target),
            )?;
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
            create_fuel_executable(&tmp_home.join(".local/bin/forc"))?;

            create_fuel_executable(&tmp_home.join(".local/bin/forc-wallet"))?;
            create_fuel_executable(&tmp_home.join(".fuelup/bin/forc-wallet"))?;

            fs::create_dir_all(tmp_home.join(".cargo/bin"))?;

            create_fuel_executable(&tmp_home.join(".cargo/bin/forc-explore"))?;
            create_fuel_executable(&tmp_home.join(".fuelup/bin/forc-explore"))?;

            create_fuel_executable(&tmp_home.join(".cargo/bin/fuel-core"))?;
        }
        FuelupState::NightlyInstalled => {
            setup_toolchain(&tmp_fuelup_root_path, &nightly)?;
            setup_settings_file(&tmp_fuelup_root_path, &nightly)?;
        }
        FuelupState::NightlyDateInstalled => {
            setup_toolchain(
                &tmp_fuelup_root_path,
                &format!("nightly-{}-{}", DATE, target),
            )?;
            setup_settings_file(
                &tmp_fuelup_root_path,
                &format!("nightly-{}-{}", DATE, target),
            )?;
        }
        FuelupState::LatestAndCustomInstalled => {
            setup_toolchain(&tmp_fuelup_root_path, &latest)?;
            setup_toolchain(&tmp_fuelup_root_path, "my-toolchain")?;
            setup_settings_file(&tmp_fuelup_root_path, &latest)?;
        }
        FuelupState::LatestAndNightlyInstalled => {
            setup_toolchain(&tmp_fuelup_root_path, &latest)?;
            setup_toolchain(&tmp_fuelup_root_path, &nightly)?;
            setup_settings_file(&tmp_fuelup_root_path, &latest)?;
        }
        FuelupState::NightlyAndNightlyDateInstalled => {
            setup_toolchain(&tmp_fuelup_root_path, &nightly)?;
            setup_toolchain(
                &tmp_fuelup_root_path,
                &format!("nightly-{}-{}", DATE, target),
            )?;
            setup_settings_file(&tmp_fuelup_root_path, &nightly)?;
        }
        FuelupState::Beta1Installed => {
            setup_toolchain(&tmp_fuelup_root_path, &beta_1)?;
            setup_toolchain(
                &tmp_fuelup_root_path,
                &format!("beta-1-{}-{}", DATE, target),
            )?;
            setup_settings_file(&tmp_fuelup_root_path, &beta_1)?;
        }
        FuelupState::LatestAndNightlyWithBetaOverride => {
            setup_toolchain(&tmp_fuelup_root_path, &latest)?;
            setup_toolchain(&tmp_fuelup_root_path, &nightly)?;
            setup_settings_file(&tmp_fuelup_root_path, &latest)?;
            setup_override_file(ToolchainOverride {
                cfg: OverrideCfg::new(ToolchainCfg { channel: beta_1 }, None),
                path: tmp_home.join(FUEL_TOOLCHAIN_TOML_FILE),
            })?;
        }
        FuelupState::LatestAndCustomWithCustomOverride => {
            setup_toolchain(&tmp_fuelup_root_path, &latest)?;
            setup_toolchain(&tmp_fuelup_root_path, "my-toolchain")?;
            setup_settings_file(&tmp_fuelup_root_path, &latest)?;
            setup_override_file(ToolchainOverride {
                cfg: OverrideCfg::new(ToolchainCfg { channel: beta_1 }, None),
                path: tmp_home.join(FUEL_TOOLCHAIN_TOML_FILE),
            })?;
        }
    }

    f(&mut TestCfg::new(
        tmp_fuelup_bin_dir_path.join("fuelup"),
        root,
        tmp_home.to_path_buf(),
    ));

    Ok(())
}
