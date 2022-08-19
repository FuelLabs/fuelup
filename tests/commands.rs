use anyhow::Result;
use fuelup::toolchain::TargetTriple;
use std::{env, path::Path};

mod testcfg;

use testcfg::FuelupState;

fn expect_files_exist(dir: &Path, expected: &mut [&str]) {
    let mut actual: Vec<String> = dir
        .read_dir()
        .expect("Could not read directory")
        .into_iter()
        .map(|b| b.unwrap().file_name().to_string_lossy().to_string())
        .collect();

    actual.sort();
    expected.sort();
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
fn fuelup_toolchain_install() -> Result<()> {
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

            expect_files_exist(
                &toolchain_dir.path().join("bin"),
                &mut [
                    "forc",
                    "forc-explore",
                    "fuel-core",
                    "forc-lsp",
                    "forc-fmt",
                    "forc-run",
                    "forc-deploy",
                ],
            );

            let output = cfg.fuelup(&["check"]);
            assert!(output.stdout.contains("forc - Up to date"));
            // TODO: uncomment once new fuel-core is released and this works
            // assert!(stdout.contains("fuel-core - Up to date"));
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
        let output = cfg.fuelup(&["toolchain", "new", "latest"]);
        let expected_stderr = "error: Invalid value \"latest\" for '<NAME>': Cannot use official toolchain name 'latest' as a custom toolchain name\n\nFor more information try --help\n";
        assert_eq!(output.stderr, expected_stderr);
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
        expect_files_exist(
            &cfg.toolchain_bin_dir("my_toolchain"),
            &mut [
                "forc",
                "forc-explore",
                "forc-lsp",
                "forc-fmt",
                "forc-run",
                "forc-deploy",
            ],
        );

        let _ = cfg.fuelup(&["component", "add", "fuel-core@0.9.5"]);
        expect_files_exist(
            &cfg.toolchain_bin_dir("my_toolchain"),
            &mut [
                "forc",
                "forc-explore",
                "fuel-core",
                "forc-lsp",
                "forc-fmt",
                "forc-run",
                "forc-deploy",
            ],
        );
    })?;

    Ok(())
}

#[test]
fn fuelup_component_add_disallowed() -> Result<()> {
    testcfg::setup(FuelupState::LatestToolchainInstalled, &|cfg| {
        let output = cfg.fuelup(&["component", "add", "forc@0.19.1"]);
        let expected_stdout = r#"Installing specific versions of components is reserved for custom toolchains.
You are currently using 'latest'.

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

        expect_files_exist(
            &latest_toolchain_bin_dir,
            &mut ["forc", "forc-explore", "fuel-core", "forc-lsp", "forc-fmt"],
        );
        let _ = cfg.fuelup(&["component", "remove", "forc"]);
        expect_files_exist(&latest_toolchain_bin_dir, &mut ["fuel-core"]);

        let _ = cfg.fuelup(&["component", "remove", "fuel-core"]);
        expect_files_exist(&latest_toolchain_bin_dir, &mut []);
    })?;

    Ok(())
}
