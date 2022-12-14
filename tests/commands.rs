use anyhow::Result;
use fuelup::{channel, fmt::format_toolchain_with_target, target_triple::TargetTriple};
use std::{env, path::Path};

mod testcfg;

use testcfg::{FuelupState, ALL_BINS, DATE};

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
        let output = cfg.fuelup(&["toolchain", "install", "latest"]);
        assert!(output.status.success());

        for entry in cfg.toolchains_dir().read_dir().expect("Could not read dir") {
            let toolchain_dir = entry.unwrap();
            let expected_toolchain_name =
                "latest-".to_owned() + &TargetTriple::from_host().unwrap().to_string();
            assert_eq!(
                expected_toolchain_name,
                toolchain_dir.file_name().to_str().unwrap()
            );
            assert!(toolchain_dir.file_type().unwrap().is_dir());
        }
    })?;

    Ok(())
}

#[test]
fn fuelup_toolchain_install_nightly() -> Result<()> {
    testcfg::setup(FuelupState::Empty, &|cfg| {
        let output = cfg.fuelup(&["toolchain", "install", "nightly"]);
        assert!(output.status.success());

        for entry in cfg.toolchains_dir().read_dir().expect("Could not read dir") {
            let toolchain_dir = entry.unwrap();
            let expected_toolchain_name =
                "nightly-".to_owned() + &TargetTriple::from_host().unwrap().to_string();
            assert_eq!(
                expected_toolchain_name,
                toolchain_dir.file_name().to_str().unwrap()
            );
            assert!(toolchain_dir.file_type().unwrap().is_dir());
        }
    })?;

    Ok(())
}

#[test]
fn fuelup_toolchain_install_nightly_date() -> Result<()> {
    testcfg::setup(FuelupState::Empty, &|cfg| {
        cfg.fuelup(&["toolchain", "install", "nightly-2022-08-31"]);

        for entry in cfg.toolchains_dir().read_dir().expect("Could not read dir") {
            let toolchain_dir = entry.unwrap();
            let expected_toolchain_name =
                "nightly-2022-08-31-".to_owned() + &TargetTriple::from_host().unwrap().to_string();
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
fn fuelup_toolchain_install_malformed_date() -> Result<()> {
    testcfg::setup(FuelupState::Empty, &|cfg| {
        let toolchain = "nightly-2022-08-31-";
        let output = cfg.fuelup(&["toolchain", "install", toolchain]);

        let expected_stdout = format!("Invalid toolchain metadata within input '{toolchain}' - You specified target '': specifying a target is not supported yet.\n");

        assert!(output.status.success());
        assert_eq!(output.stdout, expected_stdout);
    })?;

    Ok(())
}

#[test]
fn fuelup_toolchain_install_date_target_disallowed() -> Result<()> {
    testcfg::setup(FuelupState::Empty, &|cfg| {
        let toolchain = "nightly-2022-08-31-x86_64-apple-darwin";
        let output = cfg.fuelup(&["toolchain", "install", toolchain]);

        let expected_stdout =
            format!("Invalid toolchain metadata within input '{toolchain}' - You specified target 'x86_64-apple-darwin': specifying a target is not supported yet.\n");

        assert!(output.status.success());
        assert_eq!(output.stdout, expected_stdout);
    })?;

    Ok(())
}

#[test]
fn fuelup_update() -> Result<()> {
    testcfg::setup(FuelupState::LatestToolchainInstalled, &|cfg| {
        let output = cfg.fuelup(&["update"]);
        assert!(output.status.success());
        assert!(!output.stdout.contains("warning:"));

        for entry in cfg.toolchains_dir().read_dir().expect("Could not read dir") {
            let toolchain_dir = entry.unwrap();
            let expected_toolchain_name =
                "latest-".to_owned() + &TargetTriple::from_host().unwrap().to_string();
            assert_eq!(
                expected_toolchain_name,
                toolchain_dir.file_name().to_str().unwrap()
            );
            assert!(toolchain_dir.file_type().unwrap().is_dir());
        }
    })?;

    Ok(())
}

#[cfg(unix)]
#[test]
fn fuelup_update_conflict() -> Result<()> {
    testcfg::setup(FuelupState::FuelupUpdateConflict, &|cfg| {
        let output = cfg.fuelup(&["update"]);

        // There are 4 scenarios of warning messages:
        // 1) duplicate fuel executable found in PATH but not in CARGO_HOME and not in FUELUP_HOME
        let has_duplicate_message = &format!(
            "warning: 'forc' found in PATH at {}. This will take precedence over 'forc' to be installed at {}.",
            cfg.home.join(".local/bin/forc").display(),
            cfg.home.join(".fuelup/bin/forc").display()
        );
        // 2) duplicate fuel executable found in PATH and in CARGO_HOME but not in FUELUP_HOME.
        //    `cargo uninstall` is a possible action and therefore a suggestion by fuelup.
        let has_duplicate_cargo_uninstall_message =
            &format!("warning: 'fuel-core' found in PATH at {}. This will take precedence over 'fuel-core' to be installed at {}. You may want to execute 'cargo uninstall fuel-core'.\n",
                cfg.home.join(".cargo/bin/fuel-core").display(),
                cfg.home.join(".fuelup/bin/fuel-core").display()
            );
        // 3) duplicate fuel executable found in PATH and in FUELUP_HOME but not in CARGO_HOME.
        //    fuelup's version is overshadowed by the duplicate.
        let has_duplicate_overshadow_message = &format!("warning: 'forc-wallet' found in PATH at {}. This will take precedence over 'forc-wallet', already installed at {}. Consider uninstalling {}, or re-arranging your PATH to give fuelup priority.",
                cfg.home.join(".local/bin/forc-wallet").display(),
                cfg.home.join(".fuelup/bin/forc-wallet").display(),
                cfg.home.join(".local/bin/forc-wallet").display(),
            );
        // 4) duplicate fuel executable found in PATH and in FUELUP_HOME and CARGO_HOME.
        //    fuelup's version is overshadowed by the duplicate. `cargo uninstall` is a possible
        //    action and therefore a suggestion by fuelup.
        let has_duplicate_overshadow_cargo_uninstall_message = &format!("warning: 'forc-explore' found in PATH at {}. This will take precedence over 'forc-explore', already installed at {}. Consider uninstalling {}, or re-arranging your PATH to give fuelup priority. You may want to execute 'cargo uninstall forc-explore'.",
                cfg.home.join(".cargo/bin/forc-explore").display(),
                cfg.home.join(".fuelup/bin/forc-explore").display(),
                cfg.home.join(".cargo/bin/forc-explore").display(),
            );

        assert!(output.status.success());
        assert!(output.stdout.contains(has_duplicate_message));
        assert!(output
            .stdout
            .contains(has_duplicate_cargo_uninstall_message));
        assert!(output.stdout.contains(has_duplicate_overshadow_message));
        assert!(output
            .stdout
            .contains(has_duplicate_overshadow_cargo_uninstall_message));
    })?;

    Ok(())
}

#[test]
fn fuelup_toolchain_uninstall() -> Result<()> {
    testcfg::setup(FuelupState::Empty, &|cfg| {
        let toolchains = ["latest", "nightly", &format!("nightly-{}", DATE)];
        for toolchain in toolchains {
            let toolchain_with_target = format_toolchain_with_target(toolchain);
            let output = cfg.fuelup(&["toolchain", "uninstall", toolchain]);
            let expected_stdout = format!("toolchain '{}' does not exist\n", toolchain_with_target);

            assert_eq!(output.stdout, expected_stdout);
        }
    })?;

    testcfg::setup(FuelupState::AllInstalled, &|cfg| {
        let toolchains = ["latest", "nightly", &format!("nightly-{}", DATE)];
        for toolchain in toolchains {
            let toolchain_with_target = format_toolchain_with_target(toolchain);
            let output = cfg.fuelup(&["toolchain", "uninstall", toolchain]);
            let expected_stdout = format!("toolchain '{}' uninstalled\n", toolchain_with_target);

            assert!(output.stdout.contains(&expected_stdout));
        }
    })?;

    Ok(())
}

#[test]
fn fuelup_toolchain_uninstall_active_switches_default() -> Result<()> {
    testcfg::setup(FuelupState::LatestAndCustomInstalled, &|cfg| {
        cfg.fuelup(&["toolchain", "uninstall", "latest"]);
        let stdout = cfg.fuelup(&["default"]).stdout;

        assert_eq!(stdout, "my-toolchain (default)\n")
    })?;

    Ok(())
}

#[test]
fn fuelup_check() -> Result<()> {
    let latest = format!("latest-{}\n", TargetTriple::from_host().unwrap());
    let beta_1 = format!("beta-1-{}\n", TargetTriple::from_host().unwrap());
    let forc = "forc -";
    let fuel_core = "fuel-core -";
    let fuel_indexer = "fuel-indexer -";
    testcfg::setup(FuelupState::Empty, &|cfg| {
        let output = cfg.fuelup(&["check"]);
        assert!(!output.stdout.contains(&latest));
        assert!(!output.stdout.contains(&forc));
        assert!(!output.stdout.contains(&fuel_core));
        assert!(!output.stdout.contains(&fuel_indexer));
    })?;

    // Test that only the 'latest' toolchain shows.
    testcfg::setup(FuelupState::LatestAndCustomInstalled, &|cfg| {
        let output = cfg.fuelup(&["check"]);
        assert!(output.stdout.contains(&latest));
        assert!(output.stdout.contains(&forc));
        assert!(output.stdout.contains(&fuel_core));
        assert!(output.stdout.contains(&fuel_indexer));
    })?;

    // Test that toolchain names with '-' inside are parsed correctly.
    testcfg::setup(FuelupState::Beta1Installed, &|cfg| {
        let output = cfg.fuelup(&["check"]);
        assert!(output.stdout.contains(&beta_1));
        assert!(output.stdout.contains(&forc));
        assert!(output.stdout.contains(&fuel_core));
        assert!(!output.stdout.contains(&fuel_indexer));
    })?;

    Ok(())
}

#[test]
fn fuelup_show() -> Result<()> {
    testcfg::setup(FuelupState::AllInstalled, &|cfg| {
        cfg.fuelup(&["toolchain", "new", "my_toolchain"]);
        let stdout = cfg.fuelup(&["show"]).stdout;

        let mut lines = stdout.lines();
        assert_eq!(
            lines.next().unwrap(),
            &format!("Default host: {}", TargetTriple::from_host().unwrap())
        );
        assert!(lines.next().unwrap().contains("fuelup home: "));

        let target = TargetTriple::from_host().unwrap();
        let expected_stdout = &format!(
            r#"
installed toolchains
--------------------
latest-{target}
nightly-{target}
my_toolchain (default)
nightly-2022-08-30-{target}

active toolchain
-----------------
my_toolchain (default)
  forc - not found
    - forc-client
      - forc-deploy - not found
      - forc-run - not found
    - forc-doc - not found
    - forc-explore - not found
    - forc-fmt - not found
    - forc-index - not found
    - forc-lsp - not found
    - forc-wallet - not found
  fuel-core - not found
  fuel-indexer - not found
"#
        );
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
    let latest = format_toolchain_with_target("latest");
    testcfg::setup(FuelupState::LatestToolchainInstalled, &|cfg| {
        let output = cfg.fuelup(&["default"]);
        let expected_stdout = format!("{} (default)\n", latest);

        assert_eq!(output.stdout, expected_stdout);
    })?;

    Ok(())
}

#[test]
fn fuelup_default_latest_and_custom() -> Result<()> {
    testcfg::setup(FuelupState::LatestAndCustomInstalled, &|cfg| {
        let output = cfg.fuelup(&["default", "latest"]);
        let expected_stdout = format!(
            "default toolchain set to 'latest-{}'\n",
            TargetTriple::from_host().unwrap()
        );

        assert_eq!(output.stdout, expected_stdout);

        let output = cfg.fuelup(&["default", "my-toolchain"]);
        let expected_stdout = "default toolchain set to 'my-toolchain'\n";

        assert_eq!(output.stdout, expected_stdout);
    })?;

    Ok(())
}

#[test]
fn fuelup_default_uninstalled_toolchain() -> Result<()> {
    testcfg::setup(FuelupState::LatestToolchainInstalled, &|cfg| {
        let output = cfg.fuelup(&["default", "nightly"]);
        let expected_stdout = format!(
            "Toolchain with name 'nightly-{}' does not exist\n",
            TargetTriple::from_host().unwrap()
        );

        assert_eq!(output.stdout, expected_stdout);
    })?;

    Ok(())
}

#[test]
fn fuelup_default_nightly() -> Result<()> {
    testcfg::setup(FuelupState::LatestAndNightlyInstalled, &|cfg| {
        let output = cfg.fuelup(&["default", "nightly"]);
        let expected_stdout = format!(
            "default toolchain set to 'nightly-{}'\n",
            TargetTriple::from_host().unwrap()
        );

        assert_eq!(output.stdout, expected_stdout);
    })?;

    Ok(())
}

#[test]
fn fuelup_default_nightly_and_nightly_date() -> Result<()> {
    testcfg::setup(FuelupState::NightlyAndNightlyDateInstalled, &|cfg| {
        let output = cfg.fuelup(&["default", "nightly"]);
        let expected_stdout = format!(
            "default toolchain set to 'nightly-{}'\n",
            TargetTriple::from_host().unwrap()
        );
        assert_eq!(output.stdout, expected_stdout);

        let output = cfg.fuelup(&["default", &format!("nightly-{}", DATE)]);
        let expected_stdout = format!(
            "default toolchain set to 'nightly-{}-{}'\n",
            DATE,
            TargetTriple::from_host().unwrap()
        );
        assert_eq!(output.stdout, expected_stdout);
    })?;

    Ok(())
}

#[test]
fn fuelup_toolchain_new() -> Result<()> {
    testcfg::setup(FuelupState::Empty, &|cfg| {
        let name = "my-toolchain";
        let output = cfg.fuelup(&["toolchain", "new", name]);
        let expected_stdout = format!(
            "New toolchain initialized: {name}
default toolchain set to '{name}'\n"
        );

        assert_eq!(output.stdout, expected_stdout);
        assert!(cfg.toolchain_bin_dir(name).is_dir());
        let default = cfg.default_toolchain();
        assert_eq!(default, Some(name.to_string()));
    })?;

    Ok(())
}

#[test]
fn fuelup_toolchain_new_disallowed() -> Result<()> {
    testcfg::setup(FuelupState::Empty, &|cfg| {
        for toolchain in [channel::LATEST, channel::NIGHTLY] {
            let output = cfg.fuelup(&["toolchain", "new", toolchain]);
            let expected_stderr = format!("error: Invalid value \"{toolchain}\" for '<NAME>': Cannot use distributable toolchain name '{toolchain}' as a custom toolchain name\n\nFor more information try --help\n");
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
        let expected_stderr = format!("error: Invalid value \"{toolchain_name}\" for '<NAME>': Cannot use distributable toolchain name '{toolchain_name}' as a custom toolchain name\n\nFor more information try --help\n");
        assert_eq!(output.stderr, expected_stderr);
    })?;

    Ok(())
}

#[test]
fn fuelup_component_add() -> Result<()> {
    testcfg::setup(FuelupState::Empty, &|cfg| {
        let _ = cfg.fuelup(&["toolchain", "new", "my_toolchain"]);

        let _ = cfg.fuelup(&["component", "add", "fuel-core"]);
        expect_files_exist(&cfg.toolchain_bin_dir("my_toolchain"), &["fuel-core"]);
    })?;

    Ok(())
}

#[test]
fn fuelup_component_add_with_version() -> Result<()> {
    testcfg::setup(FuelupState::Empty, &|cfg| {
        let _ = cfg.fuelup(&["toolchain", "new", "my_toolchain"]);

        let _ = cfg.fuelup(&["component", "add", "fuel-core@0.9.8"]);
        expect_files_exist(&cfg.toolchain_bin_dir("my_toolchain"), &["fuel-core"]);
    })?;

    Ok(())
}

#[test]
fn fuelup_component_add_disallowed() -> Result<()> {
    let latest = format_toolchain_with_target("latest");
    let nightly = format_toolchain_with_target("nightly");
    let nightly_date = format!("nightly-{}-{}", DATE, TargetTriple::from_host().unwrap());

    testcfg::setup(FuelupState::LatestToolchainInstalled, &|cfg| {
        let output = cfg.fuelup(&["component", "add", "forc@0.19.1"]);
        let expected_stdout = format!(
            r#"Installing specific components is reserved for custom toolchains.
You are currently using '{}'.

You may create a custom toolchain using 'fuelup toolchain new <toolchain>'.
"#,
            latest
        );
        assert_eq!(output.stdout, expected_stdout);

        let output = cfg.fuelup(&["component", "add", "fuel-core"]);
        assert_eq!(output.stdout, expected_stdout);
    })?;

    testcfg::setup(FuelupState::NightlyInstalled, &|cfg| {
        let output = cfg.fuelup(&["component", "add", "forc@.19.1"]);
        let expected_stdout = format!(
            r#"Installing specific components is reserved for custom toolchains.
You are currently using '{}'.

You may create a custom toolchain using 'fuelup toolchain new <toolchain>'.
"#,
            nightly
        );
        assert_eq!(output.stdout, expected_stdout);

        let output = cfg.fuelup(&["component", "add", "fuel-core"]);
        assert_eq!(output.stdout, expected_stdout);
    })?;

    testcfg::setup(FuelupState::NightlyDateInstalled, &|cfg| {
        let output = cfg.fuelup(&["component", "add", "forc@.19.1"]);
        let expected_stdout = format!(
            r#"Installing specific components is reserved for custom toolchains.
You are currently using '{}'.

You may create a custom toolchain using 'fuelup toolchain new <toolchain>'.
"#,
            nightly_date
        );
        assert_eq!(output.stdout, expected_stdout);

        let output = cfg.fuelup(&["component", "add", "fuel-core"]);
        assert_eq!(output.stdout, expected_stdout);
    })?;
    Ok(())
}

#[test]
fn fuelup_component_remove_disallowed() -> Result<()> {
    let latest = format_toolchain_with_target("latest");
    let nightly_date = format!("nightly-{}-{}", DATE, TargetTriple::from_host().unwrap());

    testcfg::setup(FuelupState::LatestToolchainInstalled, &|cfg| {
        let latest_toolchain_bin_dir = cfg.toolchain_bin_dir(&latest);

        expect_files_exist(&latest_toolchain_bin_dir, ALL_BINS);
        let output = cfg.fuelup(&["component", "remove", "forc"]);

        let expected_stdout = format!(
            r#"Removing specific components is reserved for custom toolchains.
You are currently using '{latest}'.

You may create a custom toolchain using 'fuelup toolchain new <toolchain>'.
"#,
        );
        assert_eq!(output.stdout, expected_stdout);
        expect_files_exist(&latest_toolchain_bin_dir, ALL_BINS);
    })?;

    testcfg::setup(FuelupState::NightlyDateInstalled, &|cfg| {
        let latest_toolchain_bin_dir = cfg.toolchain_bin_dir(&nightly_date);

        expect_files_exist(&latest_toolchain_bin_dir, ALL_BINS);
        let output = cfg.fuelup(&["component", "remove", "forc"]);

        let expected_stdout = format!(
            r#"Removing specific components is reserved for custom toolchains.
You are currently using '{}'.

You may create a custom toolchain using 'fuelup toolchain new <toolchain>'.
"#,
            nightly_date
        );
        assert_eq!(output.stdout, expected_stdout);
        expect_files_exist(&latest_toolchain_bin_dir, ALL_BINS);
    })?;
    Ok(())
}

#[test]
fn fuelup_completions() -> Result<()> {
    testcfg::setup(FuelupState::LatestToolchainInstalled, &|cfg| {
        let shells = ["zsh", "bash", "fish", "powershell", "elvish"];
        for shell in shells {
            let output = cfg.fuelup(&["completions", "--shell", shell]);

            assert!(output.status.success());
        }
    })?;

    Ok(())
}
