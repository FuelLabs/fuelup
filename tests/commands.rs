use anyhow::Result;
use fuelup::toolchain::TargetTriple;
use std::env;

mod testcfg;

use testcfg::FuelupState;

#[test]
fn fuelup_version() -> Result<()> {
    testcfg::setup(FuelupState::Empty, &|cfg| {
        let expected_version = format!("fuelup {}\n", clap::crate_version!());

        let stdout = cfg.exec_cmd(&["--version"]);

        assert_eq!(stdout, expected_version);
    })?;

    Ok(())
}

#[test]
fn fuelup_toolchain_install() -> Result<()> {
    testcfg::setup(FuelupState::Empty, &|cfg| {
        cfg.exec_cmd(&["toolchain", "install", "latest"]);

        let expected_bins = ["forc", "forc-explore", "fuel-core", "forc-lsp", "forc-fmt"];

        for entry in cfg.toolchains_dir().read_dir().expect("Could not read dir") {
            let toolchain_dir = entry.unwrap();
            let expected_toolchain_name =
                "latest-".to_owned() + &TargetTriple::from_host().unwrap().to_string();
            assert_eq!(
                expected_toolchain_name,
                toolchain_dir.file_name().to_str().unwrap()
            );
            assert!(toolchain_dir.file_type().unwrap().is_dir());

            let downloaded_bins: Vec<String> = toolchain_dir
                .path()
                .join("bin")
                .read_dir()
                .expect("Could not read toolchain bin dir")
                .into_iter()
                .map(|b| b.unwrap().file_name().to_string_lossy().to_string())
                .collect();

            assert_eq!(downloaded_bins, expected_bins);
        }
    })?;

    Ok(())
}

#[test]
fn fuelup_check() -> Result<()> {
    testcfg::setup(FuelupState::Empty, &|cfg| {
        let stdout = cfg.exec_cmd(&["check"]);
        let expected_stdout = format!("\u{1b}[0m\u{1b}[1mfuelup - \u{1b}[0m\u{1b}[0m\u{1b}[1m\u{1b}[32mUp to date\u{1b}[0m : {}\n", clap::crate_version!());

        assert_eq!(stdout, expected_stdout);
    })?;

    Ok(())
}

#[test]
fn fuelup_self_update() -> Result<()> {
    testcfg::setup(FuelupState::LatestToolchainInstalled, &|cfg| {
        let stdout = cfg.exec_cmd(&["self", "update"]);

        let expected_stdout_starts_with = "Fetching binary from";
        assert!(stdout.starts_with(expected_stdout_starts_with));
    })?;

    Ok(())
}

#[test]
fn fuelup_default_empty() -> Result<()> {
    testcfg::setup(FuelupState::Empty, &|cfg| {
        let stdout = cfg.exec_cmd(&["default"]);
        let expected_stdout =
            "No default toolchain detected. Please install or create a toolchain first.\n";

        assert_eq!(stdout, expected_stdout);
    })?;

    Ok(())
}

#[test]
fn fuelup_default() -> Result<()> {
    testcfg::setup(FuelupState::LatestToolchainInstalled, &|cfg| {
        let stdout = cfg.exec_cmd(&["default"]);
        let expected_stdout = "latest-x86_64-apple-darwin (default)\n";

        assert_eq!(stdout, expected_stdout);
    })?;

    Ok(())
}

#[test]
fn fuelup_toolchain_new_only() -> Result<()> {
    testcfg::setup(FuelupState::Empty, &|cfg| {
        let stdout = cfg.exec_cmd(&["toolchain", "new", "my_toolchain"]);
        let expected_stdout = "New toolchain initialized: my_toolchain\n";
        assert_eq!(stdout, expected_stdout);
        assert!(cfg.toolchains_dir().join("my_toolchain").is_dir());
        assert!(cfg
            .toolchains_dir()
            .join("my_toolchain")
            .join("bin")
            .is_dir());

        let stdout = cfg.exec_cmd(&["default", "my_toolchain"]);
        let expected_stdout = "default toolchain set to 'my_toolchain'\n";
        assert_eq!(stdout, expected_stdout);
    })?;

    Ok(())
}

#[test]
fn fuelup_toolchain_new_and_set_default() -> Result<()> {
    testcfg::setup(FuelupState::LatestToolchainInstalled, &|cfg| {
        let stdout = cfg.exec_cmd(&["default"]);
        let expected_stdout = "latest-x86_64-apple-darwin (default)\n";
        assert_eq!(stdout, expected_stdout);

        let stdout = cfg.exec_cmd(&["toolchain", "new", "my_toolchain"]);
        let expected_stdout = "New toolchain initialized: my_toolchain\n";
        assert_eq!(stdout, expected_stdout);

        let stdout = cfg.exec_cmd(&["default", "my_toolchain"]);
        let expected_stdout = "default toolchain set to 'my_toolchain'\n";
        assert_eq!(stdout, expected_stdout);
        assert!(cfg.toolchains_dir().join("my_toolchain").is_dir());
    })?;

    Ok(())
}
