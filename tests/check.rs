use anyhow::Result;
use fuelup::target_triple::TargetTriple;

mod testcfg;

use testcfg::FuelupState;

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
        assert!(!output.stdout.contains(forc));
        assert!(!output.stdout.contains(fuel_core));
        assert!(!output.stdout.contains(fuel_indexer));
    })?;

    // Test that only the 'latest' toolchain shows.
    testcfg::setup(FuelupState::LatestAndCustomInstalled, &|cfg| {
        let output = cfg.fuelup(&["check"]);
        assert!(output.stdout.contains(&latest));
        assert!(output.stdout.contains(forc));
        assert!(output.stdout.contains(fuel_core));
        assert!(output.stdout.contains(fuel_indexer));
    })?;

    // Test that toolchain names with '-' inside are parsed correctly.
    testcfg::setup(FuelupState::Beta1Installed, &|cfg| {
        let output = cfg.fuelup(&["check"]);
        assert!(output.stdout.contains(&beta_1));
        assert!(output.stdout.contains(forc));
        assert!(output.stdout.contains(fuel_core));
        assert!(!output.stdout.contains(fuel_indexer));
    })?;

    Ok(())
}
