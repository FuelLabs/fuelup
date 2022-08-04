use anyhow::Result;
use std::{env, os::unix::prelude::CommandExt};

mod testcfg;

use testcfg::FuelupState;

#[test]
fn smoke_test() -> Result<()> {
    testcfg::setup(FuelupState::Empty, &|cfg| {
        let expected_version = format!("fuelup {}\n", clap::crate_version!());

        let output = cfg.exec_cmd(&["--version"]);
        let stdout = String::from_utf8_lossy(&output.stdout);

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
            assert_eq!("latest-x86_64-apple-darwin", toolchain_dir.file_name());
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
        let output = cfg.exec_cmd(&["check"]);
        let expected_stdout = format!("\u{1b}[0m\u{1b}[1mfuelup - \u{1b}[0m\u{1b}[0m\u{1b}[1m\u{1b}[32mUp to date\u{1b}[0m : {}\n", clap::crate_version!());
        let stdout = String::from_utf8_lossy(&output.stdout);

        assert_eq!(stdout, expected_stdout);
    })?;

    Ok(())
}

#[test]
fn fuelup_default() -> Result<()> {
    testcfg::setup(FuelupState::LatestToolchainInstalled, &|cfg| {
        let output = cfg.exec_cmd(&["default"]);
        let expected_stdout = "latest-x86_64-apple-darwin (default)\n";
        let stdout = String::from_utf8_lossy(&output.stdout);

        assert_eq!(stdout, expected_stdout);
    })?;

    Ok(())
}
