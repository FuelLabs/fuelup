use anyhow::Result;
use fuelup::settings::SettingsFile;
use fuelup::target_triple::TargetTriple;
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
    NightlyInstalled,
    NightlyDateInstalled,
    LatestAndCustomInstalled,
    LatestAndNightlyInstalled,
    NightlyAndNightlyDateInstalled,
    Beta1Installed,
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
    fs::create_dir(&tmp_fuelup_root_path.join("toolchains")).unwrap();
    fs::copy(
        root.parent().unwrap().join("fuelup"),
        &tmp_fuelup_bin_dir_path.join("fuelup"),
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
    }

    f(&mut TestCfg::new(
        tmp_fuelup_bin_dir_path.join("fuelup"),
        root,
        tmp_home.to_path_buf(),
    ));

    Ok(())
}
