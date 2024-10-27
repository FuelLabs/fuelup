mod expects;
pub mod testcfg;

use anyhow::Result;
use component::{Component, FORC};
use expects::expect_files_exist;
use fuelup::{channel, fmt::format_toolchain_with_target, target_triple::TargetTriple};
use testcfg::{yesterday, FuelupState, ALL_BINS, CUSTOM_TOOLCHAIN_NAME, DATE};

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

#[test]
fn direct_proxy_install_toolchain_in_store_forc() {
    test_direct_proxy_install_toolchain_in_store(None);
}

#[test]
fn direct_proxy_install_toolchain_in_store_publishable() {
    test_direct_proxy_install_toolchain_in_store(Some("fuel-core"));
}

#[test]
#[should_panic] // TODO: #654 will fix this
fn direct_proxy_install_toolchain_in_store_forc_plugin() {
    test_direct_proxy_install_toolchain_in_store(Some("forc-client"));
}

#[test]
#[should_panic] // TODO: #654 will fix this
fn direct_proxy_install_toolchain_in_store_forc_plugin_external() {
    test_direct_proxy_install_toolchain_in_store(Some("forc-tx"));
}

#[test]
fn direct_proxy_install_toolchain_in_store_not_forc_plugin() {
    test_direct_proxy_install_toolchain_in_store(Some("forc-wallet"));
}

#[test]
fn direct_proxy_install_toolchain_not_in_store_forc() {
    test_direct_proxy_install_toolchain_not_in_store(None);
}

#[test]
fn direct_proxy_install_toolchain_not_in_store_publishable() {
    test_direct_proxy_install_toolchain_not_in_store(Some("fuel-core"));
}

#[test]
#[should_panic] // TODO: #654 will fix this
fn direct_proxy_install_toolchain_not_in_store_forc_plugin() {
    test_direct_proxy_install_toolchain_not_in_store(Some("forc-client"));
}

#[test]
#[should_panic] // TODO: #654 will fix this
fn direct_proxy_install_toolchain_not_in_store_forc_plugin_external() {
    test_direct_proxy_install_toolchain_not_in_store(Some("forc-tx"));
}

#[test]
fn direct_proxy_install_toolchain_not_in_store_not_forc_plugin() {
    test_direct_proxy_install_toolchain_not_in_store(Some("forc-wallet"));
}

fn test_direct_proxy_install_toolchain_in_store(component_name: Option<&str>) {
    // Test steps:
    //   - trigger direct proxy call
    //     - install override toolchain
    //   - delete toolchain but keep it in store
    //   - trigger another direct proxy call
    //     - install override toolchain from store
    //   - check executables are symlinked from the store

    let component = component_name.map(|name| Component::from_name(name).unwrap());

    testcfg::setup(FuelupState::LatestToolchainInstalled, &|cfg| {
        testcfg::setup_default_override_file(cfg, component_name);

        // trigger direct_proxy install with toolchain override
        let executable = component
            .as_ref()
            .map(|c| c.executables.first().unwrap().clone())
            .unwrap_or_else(|| FORC.to_string());

        // trigger direct_proxy install with toolchain override
        cfg.exec(&executable, &["--version"]);

        // delete toolchain but keep it in store
        testcfg::delete_default_toolchain_override_toolchain(cfg);

        // trigger direct_proxy install with toolchain override already in store
        cfg.exec(&executable, &["--version"]);

        testcfg::verify_default_toolchain_override_toolchain_executables(cfg, component.as_ref());
    })
    .unwrap();
}

fn test_direct_proxy_install_toolchain_not_in_store(component_name: Option<&str>) {
    // Test steps:
    //   - trigger direct proxy call
    //     - install override toolchain
    //   - check executables are symlinked from the store

    let component = component_name.map(|name| Component::from_name(name).unwrap());

    testcfg::setup(FuelupState::LatestToolchainInstalled, &|cfg| {
        testcfg::setup_default_override_file(cfg, component_name);

        // trigger direct_proxy install with toolchain override
        let executable = component
            .as_ref()
            .map(|c| c.executables.first().unwrap().clone())
            .unwrap_or_else(|| FORC.to_string());

        cfg.exec(&executable, &["--version"]);

        testcfg::verify_default_toolchain_override_toolchain_executables(cfg, component.as_ref());
    })
    .unwrap();
}
