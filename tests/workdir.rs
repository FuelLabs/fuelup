use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use anyhow::Result;
use tempfile::tempdir_in;

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

/// Setup an empty work directory and return a command pointing to the ripgrep
/// executable whose CWD is set to the work directory.
///
/// The name given will be used to create the directory. Generally, it should
/// correspond to the test name.
pub fn setup(f: &dyn Fn(&mut TestCfg)) -> Result<()> {
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
    fs::copy(bin, &tmp_fuelup_bin_dir_path.join("fuelup"))?;

    println!("fuelup root: {:?}", tmp_fuelup_root_path);

    for entry in tmp_fuelup_root_path.read_dir()? {
        println!("fuelup entry: {:?}", entry);
    }
    env::set_var("HOME", tmp_home);
    let cmd = Command::new(tmp_fuelup_bin_dir_path.join("fuelup"));
    f(&mut TestCfg::new(cmd, root, tmp_home.to_path_buf()));
    Ok(())
}
