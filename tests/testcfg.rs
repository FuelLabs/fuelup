use anyhow::Result;
use std::{
    env, fs, io,
    path::{Path, PathBuf},
    process::Command,
};
use tempfile::tempdir_in;

pub enum FuelupState {
    Empty,
    LatestToolchainInstalled,
}

pub struct TestCfg {
    pub bin: PathBuf,
    pub root: PathBuf,
    pub home: PathBuf,
}

impl TestCfg {
    pub fn new(bin: PathBuf, root: PathBuf, home: PathBuf) -> Self {
        Self { bin, root, home }
    }

    pub fn toolchains_dir(&self) -> PathBuf {
        return self.home.join(".fuelup").join("toolchains");
    }

    pub fn exec_cmd(&mut self, args: &[&str]) -> String {
        let output = Command::new(&self.bin)
            .args(args)
            .env("HOME", &self.home)
            .output()
            .expect("Failed to execute command");
        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout.to_string()
    }
}

pub fn copy_binary<A, B>(a: A, b: B) -> std::io::Result<()>
where
    A: AsRef<Path>,
    B: AsRef<Path>,
{
    fn inner(a: &Path, b: &Path) -> io::Result<()> {
        match fs::remove_file(b) {
            Err(e) if e.kind() != io::ErrorKind::NotFound => return Err(e),
            _ => {}
        }
        fs::copy(a, b).map(drop)
    }

    inner(a.as_ref(), b.as_ref())
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
    copy_binary(&bin, &tmp_fuelup_bin_dir_path.join("fuelup"))?;

    match state {
        FuelupState::Empty => {}
        FuelupState::LatestToolchainInstalled => {
            let bin_dir = tmp_fuelup_root_path
                .join("toolchains")
                .join("latest-x86_64-apple-darwin")
                .join("bin");
            fs::create_dir_all(&bin_dir).expect("Failed");

            fs::File::create(&bin_dir.join("forc"))?;
            fs::File::create(&bin_dir.join("forc-fmt"))?;
            fs::File::create(&bin_dir.join("forc-lsp"))?;
            fs::File::create(&bin_dir.join("forc-explore"))?;
            fs::File::create(&bin_dir.join("fuel-core"))?;
            fs::copy(
                &env::current_dir()
                    .unwrap()
                    .join("tests/settings-example.toml"),
                &tmp_fuelup_root_path.join("settings.toml"),
            )
            .expect("Failed to copy settings");
        }
    }

    f(&mut TestCfg::new(
        tmp_fuelup_bin_dir_path.join("fuelup"),
        root,
        tmp_home.to_path_buf(),
    ));

    Ok(())
}
