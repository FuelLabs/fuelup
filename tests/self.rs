use anyhow::Result;

pub mod testcfg;
use testcfg::FuelupState;

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
