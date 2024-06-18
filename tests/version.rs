pub mod testcfg;

use anyhow::Result;
use std::env;
use testcfg::FuelupState;

#[test]
fn fuelup_version() -> Result<()> {
    testcfg::setup(FuelupState::Empty, &|cfg| {
        let expected_version = format!("fuelup {}\n", clap::crate_version!());
        let stdout = cfg.fuelup(&["--version"]).stdout;
        assert_eq!(stdout, expected_version);
    })?;
    Ok(())
}
