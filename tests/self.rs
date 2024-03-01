use anyhow::Result;
use testcfg::FuelupState;

pub mod testcfg;

#[test]
fn test_self_uninstall() -> Result<()> {
    testcfg::setup(FuelupState::NightlyDateInstalled, &|cfg| {
        assert!(cfg.home.exists());
        assert!(cfg.fuelup_bin_dirpath.exists());
        assert!(cfg.fuelup_path.exists());
        let output = cfg.fuelup(&["self", "uninstall", "--force"]);
        for expected in ["fuelup home", "fuelup bin"] {
            let expected = format!("removing {}", expected);
            assert!(output.stdout.contains(&expected));
        }
        assert!(cfg.home.exists());
        assert!(!cfg.fuelup_bin_dirpath.exists());
        assert!(!cfg.fuelup_path.exists());
    })?;
    Ok(())
}

#[test]
fn fuelup_self_update() -> Result<()> {
    testcfg::setup(FuelupState::NightlyDateInstalled, &|cfg| {
        let output = cfg.fuelup(&["self", "update", "--force"]);
        let expected_stdout_starts_with = "Fetching binary from";
        assert!(output.stdout.starts_with(expected_stdout_starts_with));
    })?;

    Ok(())
}

#[test]
fn fuelup_self_update_latest() -> Result<()> {
    testcfg::setup(FuelupState::LatestToolchainInstalled, &|cfg| {
        let output = cfg.fuelup(&["self", "update"]);
        let expected_stdout_starts_with = "Already up to date";
        assert!(output.stdout.contains(expected_stdout_starts_with));
    })?;

    Ok(())
}
