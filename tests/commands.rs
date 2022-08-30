use anyhow::Result;
use fuelup::{channel, target_triple::TargetTriple};
use std::{env, path::Path};

mod testcfg;

use testcfg::{FuelupState, ALL_BINS, FUEL_CORE_BIN};

use crate::testcfg::FORC_BINS;

fn expect_files_exist(dir: &Path, expected: &[&str]) {
    let mut actual: Vec<String> = dir
        .read_dir()
        .expect("Could not read directory")
        .into_iter()
        .map(|b| b.unwrap().file_name().to_string_lossy().to_string())
        .collect();

    actual.sort();
    assert_eq!(actual, expected);
}

#[test]
fn fuelup_version() -> Result<()> {
    testcfg::setup(FuelupState::Empty, &|cfg| {
        let expected_version = format!("fuelup {}\n", clap::crate_version!());

        let stdout = cfg.fuelup(&["--version"]).stdout;

        assert_eq!(stdout, expected_version);
    })?;

    Ok(())
}

#[test]
fn fuelup_toolchain_install_latest() -> Result<()> {
    testcfg::setup(FuelupState::Empty, &|cfg| {
        cfg.fuelup(&["toolchain", "install", "latest"]);

        for entry in cfg.toolchains_dir().read_dir().expect("Could not read dir") {
            let toolchain_dir = entry.unwrap();
            let expected_toolchain_name =
                "latest-".to_owned() + &TargetTriple::from_host().unwrap().to_string();
            assert_eq!(
                expected_toolchain_name,
                toolchain_dir.file_name().to_str().unwrap()
            );
            assert!(toolchain_dir.file_type().unwrap().is_dir());

            expect_files_exist(&toolchain_dir.path().join("bin"), ALL_BINS);

            let output = cfg.fuelup(&["check"]);
            assert!(output.stdout.contains("forc - Up to date"));
            // TODO: uncomment once new fuel-core is released and this works
            // assert!(stdout.contains("fuel-core - Up to date"));
        }
    })?;

    Ok(())
}

#[test]
fn fuelup_toolchain_install_nightly() -> Result<()> {
    testcfg::setup(FuelupState::Empty, &|cfg| {
        cfg.fuelup(&["toolchain", "install", "nightly"]);

        for entry in cfg.toolchains_dir().read_dir().expect("Could not read dir") {
            let toolchain_dir = entry.unwrap();
            let expected_toolchain_name =
                "nightly-".to_owned() + &TargetTriple::from_host().unwrap().to_string();
            assert_eq!(
                expected_toolchain_name,
                toolchain_dir.file_name().to_str().unwrap()
            );
            assert!(toolchain_dir.file_type().unwrap().is_dir());

            expect_files_exist(&toolchain_dir.path().join("bin"), ALL_BINS);
        }
    })?;

    Ok(())
}

#[test]
fn fuelup_check() -> Result<()> {
    testcfg::setup(FuelupState::Empty, &|cfg| {
        let output = cfg.fuelup(&["check"]);
        assert!(output.status.success());
    })?;

    Ok(())
}

#[test]
fn fuelup_show() -> Result<()> {
    testcfg::setup(FuelupState::Empty, &|cfg| {
        cfg.fuelup(&["toolchain", "new", "my_toolchain"]);
        let stdout = cfg.fuelup(&["show"]).stdout;

        let mut lines = stdout.lines();
        assert_eq!(
            lines.next().unwrap(),
            &format!("Default host: {}", TargetTriple::from_host().unwrap())
        );
        assert!(lines.next().unwrap().contains("fuelup home: "));

        let expected_stdout = r#"installed toolchains
--------------------
my_toolchain (default)

active toolchain
----------------
my_toolchain (default)
  forc - not found
    - forc-fmt - not found
    - forc-lsp - not found
    - forc-explore - not found
    - forc-client
      - forc-run - not found
      - forc-deploy - not found
  fuel-core - not found
"#;
        assert!(stdout.contains(expected_stdout));
    })?;
    Ok(())
}

#[test]
fn fuelup_self_update() -> Result<()> {
    testcfg::setup(FuelupState::LatestToolchainInstalled, &|cfg| {
        let output = cfg.fuelup(&["self", "update"]);

        let expected_stdout_starts_with = "Fetching binary from";
        assert!(output.stdout.starts_with(expected_stdout_starts_with));
    })?;

    Ok(())
}

#[test]
fn fuelup_default_empty() -> Result<()> {
    testcfg::setup(FuelupState::Empty, &|cfg| {
        let output = cfg.fuelup(&["default"]);
        let expected_stdout =
            "No default toolchain detected. Please install or create a toolchain first.\n";

        assert_eq!(output.stdout, expected_stdout);
    })?;

    Ok(())
}

#[test]
fn fuelup_default() -> Result<()> {
    testcfg::setup(FuelupState::LatestToolchainInstalled, &|cfg| {
        let output = cfg.fuelup(&["default"]);
        let expected_stdout = "latest-x86_64-apple-darwin (default)\n";

        assert_eq!(output.stdout, expected_stdout);
    })?;

    Ok(())
}

#[test]
fn fuelup_toolchain_new() -> Result<()> {
    testcfg::setup(FuelupState::Empty, &|cfg| {
        let output = cfg.fuelup(&["toolchain", "new", "my_toolchain"]);
        let expected_stdout = "New toolchain initialized: my_toolchain\n";
        assert_eq!(output.stdout, expected_stdout);
        assert!(cfg.toolchain_bin_dir("my_toolchain").is_dir());

        let output = cfg.fuelup(&["default", "my_toolchain"]);
        let expected_stdout = "default toolchain set to 'my_toolchain'\n";
        assert_eq!(output.stdout, expected_stdout);
    })?;

    Ok(())
}

#[test]
fn fuelup_toolchain_new_disallowed() -> Result<()> {
    testcfg::setup(FuelupState::Empty, &|cfg| {
        for toolchain in [channel::LATEST, channel::NIGHTLY] {
            let output = cfg.fuelup(&["toolchain", "new", toolchain]);
            let expected_stderr = format!("error: Invalid value \"{toolchain}\" for '<NAME>': Cannot use official toolchain name '{toolchain}' as a custom toolchain name\n\nFor more information try --help\n");
            assert_eq!(output.stderr, expected_stderr);
        }
    })?;

    Ok(())
}

#[test]
fn fuelup_toolchain_new_disallowed_with_target() -> Result<()> {
    testcfg::setup(FuelupState::Empty, &|cfg| {
        let target_triple = TargetTriple::from_host().unwrap();
        let toolchain_name = "latest-".to_owned() + &target_triple.to_string();
        let output = cfg.fuelup(&["toolchain", "new", &toolchain_name]);
        let expected_stderr = format!("error: Invalid value \"{toolchain_name}\" for '<NAME>': Cannot use official toolchain name '{toolchain_name}' as a custom toolchain name\n\nFor more information try --help\n");
        assert_eq!(output.stderr, expected_stderr);
    })?;

    Ok(())
}

#[test]
fn fuelup_toolchain_new_and_set_default() -> Result<()> {
    testcfg::setup(FuelupState::LatestToolchainInstalled, &|cfg| {
        let output = cfg.fuelup(&["default"]);
        let expected_stdout = "latest-x86_64-apple-darwin (default)\n";
        assert_eq!(output.stdout, expected_stdout);
        assert!(!cfg.toolchain_bin_dir("my_toolchain").is_dir());

        let output = cfg.fuelup(&["toolchain", "new", "my_toolchain"]);
        let expected_stdout = "New toolchain initialized: my_toolchain\n";
        assert_eq!(output.stdout, expected_stdout);
        assert!(cfg.toolchain_bin_dir("my_toolchain").is_dir());

        let output = cfg.fuelup(&["default", "my_toolchain"]);
        let expected_stdout = "default toolchain set to 'my_toolchain'\n";
        assert_eq!(output.stdout, expected_stdout);
    })?;

    Ok(())
}

#[test]
fn fuelup_component_add() -> Result<()> {
    testcfg::setup(FuelupState::Empty, &|cfg| {
        let output = cfg.fuelup(&["toolchain", "new", "my_toolchain"]);
        let expected_stdout = "New toolchain initialized: my_toolchain\n";
        assert_eq!(output.stdout, expected_stdout);

        let output = cfg.fuelup(&["default", "my_toolchain"]);
        let expected_stdout = "default toolchain set to 'my_toolchain'\n";
        assert_eq!(output.stdout, expected_stdout);
        assert!(cfg.toolchain_bin_dir("my_toolchain").is_dir());

        let _ = cfg.fuelup(&["component", "add", "forc"]);
        expect_files_exist(&cfg.toolchain_bin_dir("my_toolchain"), FORC_BINS);

        let _ = cfg.fuelup(&["component", "add", "fuel-core@0.9.5"]);
        expect_files_exist(&cfg.toolchain_bin_dir("my_toolchain"), ALL_BINS);
    })?;

    Ok(())
}

#[test]
fn fuelup_component_add_disallowed() -> Result<()> {
    testcfg::setup(FuelupState::LatestToolchainInstalled, &|cfg| {
        let output = cfg.fuelup(&["component", "add", "forc@0.19.1"]);
        let expected_stdout = r#"Installing specific versions of components is reserved for custom toolchains.
You are currently using 'latest-x86_64-apple-darwin'.

You may create a custom toolchain using 'fuelup toolchain new <toolchain>'.
"#;
        assert_eq!(output.stdout, expected_stdout);
    })?;

    Ok(())
}
#[test]
fn fuelup_component_remove() -> Result<()> {
    testcfg::setup(FuelupState::LatestToolchainInstalled, &|cfg| {
        let latest_toolchain_bin_dir = cfg.toolchain_bin_dir("latest-x86_64-apple-darwin");

        expect_files_exist(&latest_toolchain_bin_dir, ALL_BINS);
        let _ = cfg.fuelup(&["component", "remove", "forc"]);
        expect_files_exist(&latest_toolchain_bin_dir, FUEL_CORE_BIN);

        let _ = cfg.fuelup(&["component", "remove", "fuel-core"]);
        expect_files_exist(&latest_toolchain_bin_dir, &[]);
    })?;

    Ok(())
}
