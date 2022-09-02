use anyhow::Result;
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
};
use tempfile::tempdir_in;

pub enum FuelupState {
    Empty,
    LatestToolchainInstalled,
    LatestAndNightlyInstalled,
    NightlyAndNightlyDateInstalled,
}

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

pub const FORC_BINS: &[&str] = &[
    "forc",
    "forc-deploy",
    "forc-explore",
    "forc-fmt",
    "forc-lsp",
    "forc-run",
];

pub static ALL_BINS: &[&str] = &[
    "forc",
    "forc-deploy",
    "forc-explore",
    "forc-fmt",
    "forc-lsp",
    "forc-run",
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

    pub fn fuelup(&mut self, args: &[&str]) -> TestOutput {
        let output = Command::new(&self.fuelup_path)
            .args(args)
            .env("HOME", &self.home)
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

fn setup_toolchain(fuelup_home_path: &Path, toolchain: &str) -> Result<()> {
    let bin_dir = fuelup_home_path
        .join("toolchains")
        .join(toolchain)
        .join("bin");
    fs::create_dir_all(&bin_dir).expect("Failed to create temporary latest toolchain bin dir");

    for bin in ALL_BINS {
        fs::File::create(&bin_dir.join(bin))?;
    }

    fs::copy(
        &env::current_dir()
            .unwrap()
            .join("tests/settings-example.toml"),
        &fuelup_home_path.join("settings.toml"),
    )
    .expect("Failed to copy settings");

    Ok(())
}

pub fn setup(state: FuelupState, f: &dyn Fn(&mut TestCfg)) -> Result<()> {
    let root = env::current_exe()
        .unwrap()
        .parent()
        .expect("fuelup's directory")
        .to_path_buf();

    let testdir = tempdir_in(&root).unwrap();
    let tmp_home = testdir.path();

    let tmp_fuelup_root_path = tmp_home.join(".fuelup");
    let tmp_fuelup_bin_dir_path = tmp_home.join(".fuelup").join("bin");
    fs::create_dir(&tmp_fuelup_root_path).unwrap();
    fs::create_dir(&tmp_fuelup_bin_dir_path).unwrap();
    fs::create_dir(&tmp_fuelup_root_path.join("toolchains")).unwrap();
    fs::copy(
        root.parent().unwrap().join("fuelup"),
        &tmp_fuelup_bin_dir_path.join("fuelup"),
    )?;

    match state {
        FuelupState::Empty => {}
        FuelupState::LatestToolchainInstalled => {
            setup_toolchain(&tmp_fuelup_root_path, "latest-x86_64-apple-darwin")?;
        }
        FuelupState::LatestAndNightlyInstalled => {
            setup_toolchain(&tmp_fuelup_root_path, "latest-x86_64-apple-darwin")?;
            setup_toolchain(&tmp_fuelup_root_path, "nightly-x86_64-apple-darwin")?;
        }
        FuelupState::NightlyAndNightlyDateInstalled => {
            setup_toolchain(&tmp_fuelup_root_path, "nightly-x86_64-apple-darwin")?;
            setup_toolchain(
                &tmp_fuelup_root_path,
                "nightly-2022-08-30-x86_64-apple-darwin",
            )?;
        }
    }

    f(&mut TestCfg::new(
        tmp_fuelup_bin_dir_path.join("fuelup"),
        root,
        tmp_home.to_path_buf(),
    ));

    Ok(())
}
