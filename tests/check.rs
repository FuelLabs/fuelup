pub mod testcfg;

use anyhow::Result;
use fuelup::target_triple::TargetTriple;
use testcfg::FuelupState;

#[test]
fn test_fuelup_check() -> Result<()> {
    let latest = format!("latest-{}", TargetTriple::from_host().unwrap());
    let testnet = format!("testnet-{}", TargetTriple::from_host().unwrap());
    let forc = "forc -";
    let fuel_core = "fuel-core -";
    let fuel_indexer = "fuel-indexer -";
    testcfg::setup(FuelupState::Empty, &|cfg| {
        let output = cfg.fuelup(&["check", "--verbose"]);
        let stripped = strip_ansi_escapes::strip(output.stdout);
        let stdout = String::from_utf8_lossy(&stripped);
        assert!(!stdout.contains(&latest));
        assert!(!stdout.contains(forc));
        assert!(!stdout.contains(fuel_core));
        assert!(!stdout.contains(fuel_indexer));
    })?;

    // Test that only the 'latest' toolchain shows.
    testcfg::setup(FuelupState::LatestAndCustomInstalled, &|cfg| {
        let output = cfg.fuelup(&["check", "--verbose"]);
        let stripped = strip_ansi_escapes::strip(output.stdout);
        let stdout = String::from_utf8_lossy(&stripped);
        assert!(stdout.contains(&latest));
        assert!(stdout.contains(forc));
        assert!(stdout.contains(fuel_core));
    })?;

    Ok(())
}
