use anyhow::Result;
use fuelup::{fmt::format_toolchain_with_target, target_triple::TargetTriple};

pub mod testcfg;
use testcfg::{FuelupState, ALL_BINS, DATE};

mod expects;
use expects::expect_files_exist;

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
