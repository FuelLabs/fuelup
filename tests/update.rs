use anyhow::Result;
use fuelup::target_triple::TargetTriple;

mod testcfg;
use testcfg::FuelupState;

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
