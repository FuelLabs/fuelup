use anyhow::Result;

mod testcfg;
use testcfg::FuelupState;

#[test]
fn fuelup_self_update() -> Result<()> {
    testcfg::setup(FuelupState::LatestToolchainInstalled, &|cfg| {
        let output = cfg.fuelup(&["self", "update"]);

        let expected_stdout_starts_with = "Fetching binary from";
        assert!(output.stdout.starts_with(expected_stdout_starts_with));
    })?;

    Ok(())
}
