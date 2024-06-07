use anyhow::Result;
use testcfg::FuelupState;

pub mod testcfg;

#[test]
fn fuelup_upgrade() -> Result<()> {
    testcfg::setup(FuelupState::LatestToolchainInstalled, &|cfg| {
        let output = cfg.fuelup(&["upgrade"]);
        let expected_stdout_starts_with = "Already up to date";
        assert!(output.stdout.contains(expected_stdout_starts_with));
    })?;

    Ok(())
}
