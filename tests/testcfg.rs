use std::{
    env,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use anyhow::Result;
use tempfile::{tempdir_in, tempfile_in};

pub enum FuelupState {
    Empty,
    LatestToolchainInstalled,
}

pub struct TestCfg {
    pub cmd: Command,
    pub root: PathBuf,
    pub home: PathBuf,
}

impl TestCfg {
    pub fn new(cmd: Command, root: PathBuf, home: PathBuf) -> Self {
        Self { cmd, root, home }
    }

    pub fn fuelup_dir(&self) -> PathBuf {
        return self.home.join(".fuelup");
    }

    pub fn toolchains_dir(&self) -> PathBuf {
        return self.home.join(".fuelup").join("toolchains");
    }

    pub fn exec_cmd(&mut self, args: &[&str]) -> Output {
        self.cmd.args(args).output().expect("Failed to run command")
    }
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

    let bin = root.parent().unwrap().join("fuelup");
    fs::copy(&bin, &tmp_fuelup_bin_dir_path.join("fuelup"))?;

    match state {
        FuelupState::Empty => {}
        FuelupState::LatestToolchainInstalled => {
            let bin_dir = tmp_fuelup_root_path
                .join("toolchains")
                .join("latest-x86_64-apple-darwin")
                .join("bin");
            fs::create_dir_all(&bin_dir).expect("Failed");

            fs::copy(
                &env::current_dir()
                    .unwrap()
                    .join("tests/settings-example.toml"),
                &tmp_fuelup_root_path.join("settings.toml"),
            )
            .expect("Failed to copy settings");
        }
    }

    env::set_var("HOME", tmp_home);
    let cmd = Command::new(tmp_fuelup_bin_dir_path.join("fuelup"));
    f(&mut TestCfg::new(cmd, root, tmp_home.to_path_buf()));
    Ok(())
}
