use anyhow::Result;
use fuelup::{
    constants::FUEL_TOOLCHAIN_TOML_FILE,
    target_triple::TargetTriple,
    toolchain_override::{self, OverrideCfg, ToolchainCfg, ToolchainOverride},
};
use std::str::FromStr;

pub mod testcfg;
use testcfg::FuelupState;

#[test]
fn fuelup_show_latest() -> Result<()> {
    testcfg::setup(FuelupState::AllInstalled, &|cfg| {
        cfg.fuelup(&["show"]);
        let stdout = cfg.fuelup(&["show"]).stdout;
        let target = TargetTriple::from_host().unwrap();

        let mut lines = stdout.lines();
        assert_eq!(lines.next().unwrap(), &format!("Default host: {target}"));
        assert!(lines.next().unwrap().contains("fuelup home: "));

        let expected_stdout = &format!(
            r#"
installed toolchains
--------------------
latest-{target} (default)
nightly-{target}
nightly-2022-08-30-{target}

active toolchain
-----------------
latest-{target} (default)
  forc : 0.1.0
    - forc-client
      - forc-deploy : 0.1.0
      - forc-run : 0.1.0
    - forc-doc : 0.1.0
    - forc-explore : 0.1.0
    - forc-fmt : 0.1.0
    - forc-index : 0.1.0
    - forc-lsp : 0.1.0
    - forc-tx : 0.1.0
    - forc-wallet : 0.1.0
  fuel-core : 0.1.0
  fuel-core-keygen - 0.1.0
  fuel-indexer : 0.1.0
"#
        );
        assert!(stdout.contains(expected_stdout));
        assert!(!stdout.contains("fuels versions"));
    })?;
    Ok(())
}

#[test]
fn fuelup_show_and_switch() -> Result<()> {
    testcfg::setup(FuelupState::AllInstalled, &|cfg| {
        cfg.fuelup(&["show"]);
        let mut stdout = cfg.fuelup(&["show"]).stdout;
        let mut target = TargetTriple::from_host().unwrap();

        let mut lines = stdout.lines();
        assert_eq!(lines.next().unwrap(), &format!("Default host: {target}"));
        assert!(lines.next().unwrap().contains("fuelup home: "));

        let expected_stdout = &format!(
            r#"
installed toolchains
--------------------
latest-{target} (default)
nightly-{target}
nightly-2022-08-30-{target}

active toolchain
-----------------
latest-{target} (default)
  forc : 0.1.0
    - forc-client
      - forc-deploy : 0.1.0
      - forc-run : 0.1.0
    - forc-doc : 0.1.0
    - forc-explore : 0.1.0
    - forc-fmt : 0.1.0
    - forc-index : 0.1.0
    - forc-lsp : 0.1.0
    - forc-tx : 0.1.0
    - forc-wallet : 0.1.0
  fuel-core : 0.1.0
  fuel-core-keygen - 0.1.0
  fuel-indexer : 0.1.0
"#
        );
        assert!(stdout.contains(expected_stdout));
        assert!(!stdout.contains("fuels versions"));

        cfg.fuelup(&["default", "nightly"]);
        stdout = cfg.fuelup(&["show"]).stdout;
        target = TargetTriple::from_host().unwrap();

        let expected_stdout = &format!(
            r#"
installed toolchains
--------------------
latest-{target}
nightly-{target} (default)
nightly-2022-08-30-{target}

active toolchain
-----------------
nightly-{target} (default)
  forc : 0.2.0
    - forc-client
      - forc-deploy : 0.2.0
      - forc-run : 0.2.0
    - forc-doc : 0.2.0
    - forc-explore : 0.2.0
    - forc-fmt : 0.2.0
    - forc-index : 0.2.0
    - forc-lsp : 0.2.0
    - forc-tx : 0.2.0
    - forc-wallet : 0.2.0
  fuel-core : 0.2.0
  fuel-core-keygen - 0.2.0
  fuel-indexer : 0.2.0
"#
        );
        assert!(stdout.contains(expected_stdout));
        assert!(!stdout.contains("fuels versions"));
    })?;

    Ok(())
}

#[test]
fn fuelup_show_custom() -> Result<()> {
    testcfg::setup(FuelupState::Empty, &|cfg| {
        cfg.fuelup(&["toolchain", "new", "my_toolchain"]);
        let stdout = cfg.fuelup(&["show"]).stdout;

        let mut lines = stdout.lines();
        assert_eq!(
            lines.next().unwrap(),
            &format!("Default host: {}", TargetTriple::from_host().unwrap())
        );
        assert!(lines.next().unwrap().contains("fuelup home: "));

        let expected_stdout = r#"
installed toolchains
--------------------
my_toolchain (default)

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
    - forc-tx - not found
    - forc-wallet - not found
  fuel-core - not found
  fuel-core-keygen - not found
  fuel-indexer - not found
"#;
        assert!(stdout.contains(expected_stdout));
        assert!(!stdout.contains("fuels versions"));
    })?;
    Ok(())
}

#[test]
fn fuelup_show_override() -> Result<()> {
    testcfg::setup(FuelupState::LatestWithBetaOverride, &|cfg| {
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
    - forc-tx - not found
    - forc-wallet - not found
  fuel-core - not found
  fuel-core-keygen - not found
  fuel-indexer - not found
"#,
            cfg.home.join(FUEL_TOOLCHAIN_TOML_FILE).display()
        );
        assert!(stdout.contains(expected_stdout));
        assert!(!stdout.contains("fuels versions"));
    })?;
    Ok(())
}

#[test]
fn fuelup_show_latest_then_override() -> Result<()> {
    testcfg::setup(FuelupState::AllInstalled, &|cfg| {
        let mut stdout = cfg.fuelup(&["show"]).stdout;

        let mut lines = stdout.lines();
        assert_eq!(
            lines.next().unwrap(),
            &format!("Default host: {}", TargetTriple::from_host().unwrap())
        );
        assert!(lines.next().unwrap().contains("fuelup home: "));

        let target = TargetTriple::from_host().unwrap();
        let expected_stdout = &format!(
            r#"installed toolchains
--------------------
latest-{target} (default)
nightly-{target}
nightly-2022-08-30-{target}

active toolchain
-----------------
latest-{target} (default)
  forc : 0.1.0
    - forc-client
      - forc-deploy : 0.1.0
      - forc-run : 0.1.0
    - forc-doc : 0.1.0
    - forc-explore : 0.1.0
    - forc-fmt : 0.1.0
    - forc-index : 0.1.0
    - forc-lsp : 0.1.0
    - forc-tx : 0.1.0
    - forc-wallet : 0.1.0
  fuel-core : 0.1.0
  fuel-indexer : 0.1.0
"#,
        );
        assert!(stdout.contains(expected_stdout));
        assert!(!stdout.contains("fuels versions"));

        let toolchain_override = ToolchainOverride {
            cfg: OverrideCfg::new(
                ToolchainCfg {
                    channel: toolchain_override::Channel::from_str("nightly-2022-08-30").unwrap(),
                },
                None,
            ),
            path: cfg.home.join(FUEL_TOOLCHAIN_TOML_FILE),
        };
        let document = toolchain_override.to_toml();
        std::fs::write(toolchain_override.path, document.to_string())
            .unwrap_or_else(|_| panic!("Failed to write {FUEL_TOOLCHAIN_TOML_FILE}"));

        stdout = cfg.fuelup(&["show"]).stdout;

        let mut lines = stdout.lines();
        while let Some(line) = lines.next() {
            if line.contains("active toolchain") {
                // Skip the header line, '-----------------'
                lines.next();
                break;
            }
        }

        // Check that active toolchain is the override and that versions changed.
        assert!(lines
            .next()
            .unwrap()
            .starts_with(&format!("nightly-2022-08-30-{target} (override), path:")));
        let expected_stdout = &r#"forc : 0.2.0
    - forc-client
      - forc-deploy : 0.2.0
      - forc-run : 0.2.0
    - forc-doc : 0.2.0
    - forc-explore : 0.2.0
    - forc-fmt : 0.2.0
    - forc-index : 0.2.0
    - forc-lsp : 0.2.0
    - forc-tx : 0.2.0
    - forc-wallet : 0.2.0
  fuel-core : 0.2.0
  fuel-indexer : 0.2.0
"#;
        assert!(stdout.contains(expected_stdout));
        assert!(!stdout.contains("fuels versions"));
    })?;
    Ok(())
}
