use anyhow::Result;
use chrono::{Duration, Utc};
use fuelup::{channel, fmt::format_toolchain_with_target, target_triple::TargetTriple};

pub mod testcfg;
use testcfg::{FuelupState, ALL_BINS, CUSTOM_TOOLCHAIN_NAME, DATE};

mod expects;
use expects::expect_files_exist;

fn yesterday() -> String {
    let current_date = Utc::now();
    let yesterday = current_date - Duration::days(1);
    yesterday.format("%Y-%m-%d").to_string()
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

        let expected_stdout = format!("Unknown name for toolchain: {toolchain}\n");

        assert!(output.status.success());
        assert_eq!(output.stdout, expected_stdout);
    })?;

    Ok(())
}

#[test]
fn fuelup_toolchain_install_date_target_allowed() -> Result<()> {
    testcfg::setup(FuelupState::Empty, &|cfg| {
        let toolchain = format!("nightly-{}-x86_64-apple-darwin", yesterday());
        let output = cfg.fuelup(&["toolchain", "install", &toolchain]);
        assert!(output.status.success());
    })?;

    Ok(())
}

#[test]
fn fuelup_toolchain_uninstall() -> Result<()> {
    testcfg::setup(FuelupState::Empty, &|cfg| {
        let toolchains = ["latest", "nightly", &format!("nightly-{DATE}")];
        for toolchain in toolchains {
            let toolchain_with_target = format_toolchain_with_target(toolchain);
            let output = cfg.fuelup(&["toolchain", "uninstall", toolchain]);
            let expected_stdout = format!("Toolchain '{toolchain_with_target}' does not exist\n");
            assert!(output.stdout.contains(&expected_stdout));
        }
    })?;

    testcfg::setup(FuelupState::AllInstalled, &|cfg| {
        let toolchains = ["latest", "nightly", &format!("nightly-{DATE}")];

        // Cannot remove the active, even if there are others to switch to
        let output = cfg.fuelup(&["toolchain", "uninstall", toolchains[0]]);
        let expected_stdout = "as it is currently the default toolchain. Run `fuelup default <toolchain>` to update the default toolchain.";
        assert!(output.stdout.contains(expected_stdout));

        for toolchain in &toolchains[1..2] {
            let toolchain_with_target = format_toolchain_with_target(toolchain);
            assert!(cfg.toolchains_dir().join(&toolchain_with_target).is_dir());
            let output = cfg.fuelup(&["toolchain", "uninstall", toolchain]);
            let expected_stdout = format!("Toolchain '{toolchain_with_target}' uninstalled\n");
            assert!(!cfg.toolchains_dir().join(toolchain_with_target).is_dir());
            assert!(
                output.stdout.contains(&expected_stdout),
                "toolchain: {}",
                toolchain
            );
        }

        // Cannot remove the active, if it is the only one
        let output = cfg.fuelup(&["toolchain", "uninstall", toolchains[0]]);
        let expected_stdout = "as it is currently the default toolchain. Run `fuelup default <toolchain>` to update the default toolchain.";
        assert!(output.stdout.contains(expected_stdout));
    })?;

    Ok(())
}

#[test]
fn fuelup_toolchain_new() -> Result<()> {
    testcfg::setup(FuelupState::Empty, &|cfg| {
        let output = cfg.fuelup(&["toolchain", "new", CUSTOM_TOOLCHAIN_NAME]);
        let expected_stdout = format!(
            "New toolchain initialized: {CUSTOM_TOOLCHAIN_NAME}
Default toolchain set to '{CUSTOM_TOOLCHAIN_NAME}'\n"
        );

        assert_eq!(output.stdout, expected_stdout);
        assert!(cfg.toolchain_bin_dir(CUSTOM_TOOLCHAIN_NAME).is_dir());
        let default = cfg.default_toolchain();
        assert_eq!(default, Some(CUSTOM_TOOLCHAIN_NAME.to_string()));
    })?;

    Ok(())
}

#[test]
fn fuelup_toolchain_new_disallowed() -> Result<()> {
    testcfg::setup(FuelupState::Empty, &|cfg| {
        for toolchain in [channel::LATEST, channel::NIGHTLY] {
            let output = cfg.fuelup(&["toolchain", "new", toolchain]);
            let expected_stderr = format!("error: invalid value '{toolchain}' for '<NAME>': Cannot use distributable toolchain name '{toolchain}' as a custom toolchain name\n\nFor more information, try '--help'.\n");
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
        let expected_stderr = format!("error: invalid value '{toolchain_name}' for '<NAME>': Cannot use distributable toolchain name '{toolchain_name}' as a custom toolchain name\n\nFor more information, try '--help'.\n");
        assert_eq!(output.stderr, expected_stderr);
    })?;

    Ok(())
}
