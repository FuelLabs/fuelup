use anyhow::Result;
use fuelup::{constants::FUEL_TOOLCHAIN_TOML_FILE, target_triple::TargetTriple};

pub mod testcfg;
use testcfg::FuelupState;

#[test]
fn fuelup_show() -> Result<()> {
    testcfg::setup(FuelupState::AllInstalled, &|cfg| {
        cfg.fuelup(&["toolchain", "new", "my_toolchain"]);
        let stdout = cfg.fuelup(&["show"]).stdout;

        let mut lines = stdout.lines();
        assert_eq!(
            lines.next().unwrap(),
            &format!("Default host: {}", TargetTriple::from_host().unwrap())
        );
        assert!(lines.next().unwrap().contains("fuelup home: "));

        let target = TargetTriple::from_host().unwrap();
        let expected_stdout = &format!(
            r#"
installed toolchains
--------------------
latest-{target}
nightly-{target}
my_toolchain (default)
nightly-2022-08-30-{target}

active toolchain
-----------------
my_toolchain (default)
  forc - not found
    - forc-client
      - forc-deploy - not found
      - forc-run - not found
    - forc-doc - not found
    - forc-explore - not found
    - forc-fmt - not found
    - forc-index - not found
    - forc-lsp - not found
    - forc-wallet - not found
  fuel-core - not found
  fuel-indexer - not found
"#
        );
        assert!(stdout.contains(expected_stdout));
    })?;
    Ok(())
}

#[test]
fn fuelup_show_override() -> Result<()> {
    testcfg::setup(FuelupState::LatestAndNightlyWithBetaOverride, &|cfg| {
        let stdout = cfg.fuelup(&["show"]).stdout;

        let mut lines = stdout.lines();
        assert_eq!(
            lines.next().unwrap(),
            &format!("Default host: {}", TargetTriple::from_host().unwrap())
        );
        assert!(lines.next().unwrap().contains("fuelup home: "));

        let target = TargetTriple::from_host().unwrap();
        let expected_stdout = &format!(
            r#"
installed toolchains
--------------------
latest-{target} (default)
nightly-{target}

active toolchain
-----------------
beta-1-{target} (override), path: {}
  forc - not found
    - forc-client
      - forc-deploy - not found
      - forc-run - not found
    - forc-doc - not found
    - forc-explore - not found
    - forc-fmt - not found
    - forc-index - not found
    - forc-lsp - not found
    - forc-wallet - not found
  fuel-core - not found
  fuel-indexer - not found
"#,
            cfg.home.join(FUEL_TOOLCHAIN_TOML_FILE).display()
        );
        assert!(stdout.contains(expected_stdout));
    })?;
    Ok(())
}
