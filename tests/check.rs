use anyhow::Result;
use fuelup::target_triple::TargetTriple;

pub mod testcfg;

use testcfg::FuelupState;

#[test]
fn fuelup_check() -> Result<()> {
    let latest = format!("latest-{}", TargetTriple::from_host().unwrap());
    let beta_1 = format!("beta-1-{}", TargetTriple::from_host().unwrap());
    let forc = "forc -";
    let fuel_core = "fuel-core -";
    let fuel_indexer = "fuel-indexer -";
    testcfg::setup(FuelupState::Empty, &|cfg| {
        let output = cfg.fuelup(&["check"]);
        let stripped = strip_ansi_escapes::strip(output.stdout);
        let stdout = String::from_utf8_lossy(&stripped);
        assert!(!stdout.contains(&latest));
        assert!(!stdout.contains(forc));
        assert!(!stdout.contains(fuel_core));
        assert!(!stdout.contains(fuel_indexer));
    })?;

    // Test that only the 'latest' toolchain shows.
    testcfg::setup(FuelupState::LatestAndCustomInstalled, &|cfg| {
        let output = cfg.fuelup(&["check"]);
        let stripped = strip_ansi_escapes::strip(output.stdout);
        let stdout = String::from_utf8_lossy(&stripped);
        println!("\n{}", &stdout);
        assert!(stdout.contains(&latest));
        assert!(stdout.contains(forc));
        assert!(stdout.contains(fuel_core));
    })?;

    // Test that toolchain names with '-' inside are parsed correctly.
    testcfg::setup(FuelupState::Beta1Installed, &|cfg| {
        let output = cfg.fuelup(&["check"]);
        let stripped = strip_ansi_escapes::strip(output.stdout);
        let stdout = String::from_utf8_lossy(&stripped);
        assert!(stdout.contains(&beta_1));
        assert!(stdout.contains(forc));
        assert!(stdout.contains(fuel_core));
        assert!(!stdout.contains(fuel_indexer));
    })?;

    Ok(())
}
