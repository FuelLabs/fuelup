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
        let stripped = strip_ansi_escapes::strip(cfg.fuelup(&["show"]).stdout);
        let stdout = String::from_utf8_lossy(&stripped);
        let target = TargetTriple::from_host().unwrap();
        let fuelup_home = cfg.fuelup_dir();
        let fuelup_home_str = fuelup_home.to_string_lossy();
        let expected_stdout = format!(
            r#"Default host: {target}
fuelup home: {fuelup_home_str}

installed toolchains
--------------------
latest-{target} (default)
nightly-{target}
nightly-2022-08-30-{target}

active toolchain
----------------
latest-{target} (default)
  forc : 0.1.0
    - forc-client
      - forc-deploy : 0.1.0
      - forc-run : 0.1.0
    - forc-crypto : 0.1.0
    - forc-debug : 0.1.0
    - forc-doc : 0.1.0
    - forc-explore : 0.1.0
    - forc-fmt : 0.1.0
    - forc-lsp : 0.1.0
    - forc-tx : 0.1.0
    - forc-wallet : 0.1.0
  fuel-core : 0.1.0
  fuel-core-keygen : not found
"#
        );

        assert_eq!(stdout.to_string(), expected_stdout);
    })?;
    Ok(())
}

#[test]
fn fuelup_show_and_switch() -> Result<()> {
    testcfg::setup(FuelupState::AllInstalled, &|cfg| {
        cfg.fuelup(&["show"]);
        let mut stripped = strip_ansi_escapes::strip(cfg.fuelup(&["show"]).stdout);
        let mut stdout = String::from_utf8_lossy(&stripped);
        let target = TargetTriple::from_host().unwrap();
        let fuelup_home = cfg.fuelup_dir();
        let fuelup_home_str = fuelup_home.to_string_lossy();
        let expected_stdout = format!(
            r#"Default host: {target}
fuelup home: {fuelup_home_str}

installed toolchains
--------------------
latest-{target} (default)
nightly-{target}
nightly-2022-08-30-{target}

active toolchain
----------------
latest-{target} (default)
  forc : 0.1.0
    - forc-client
      - forc-deploy : 0.1.0
      - forc-run : 0.1.0
    - forc-crypto : 0.1.0
    - forc-debug : 0.1.0
    - forc-doc : 0.1.0
    - forc-explore : 0.1.0
    - forc-fmt : 0.1.0
    - forc-lsp : 0.1.0
    - forc-tx : 0.1.0
    - forc-wallet : 0.1.0
  fuel-core : 0.1.0
  fuel-core-keygen : not found
"#
        );
        assert_eq!(stdout, expected_stdout);

        cfg.fuelup(&["default", "nightly"]);
        stripped = strip_ansi_escapes::strip(cfg.fuelup(&["show"]).stdout);
        stdout = String::from_utf8_lossy(&stripped);
        let fuelup_home = cfg.fuelup_dir();
        let fuelup_home_str = fuelup_home.to_string_lossy();
        let expected_stdout = format!(
            r#"Default host: {target}
fuelup home: {fuelup_home_str}

installed toolchains
--------------------
latest-{target}
nightly-{target} (default)
nightly-2022-08-30-{target}

active toolchain
----------------
nightly-{target} (default)
  forc : 0.2.0
    - forc-client
      - forc-deploy : 0.2.0
      - forc-run : 0.2.0
    - forc-crypto : 0.2.0
    - forc-debug : 0.2.0
    - forc-doc : 0.2.0
    - forc-explore : 0.2.0
    - forc-fmt : 0.2.0
    - forc-lsp : 0.2.0
    - forc-tx : 0.2.0
    - forc-wallet : 0.2.0
  fuel-core : 0.2.0
  fuel-core-keygen : not found
"#
        );
        assert_eq!(stdout, expected_stdout);
    })?;

    Ok(())
}

#[test]
fn fuelup_show_custom() -> Result<()> {
    testcfg::setup(FuelupState::Empty, &|cfg| {
        cfg.fuelup(&["toolchain", "new", "my_toolchain"]);
        let stripped = strip_ansi_escapes::strip(cfg.fuelup(&["show"]).stdout);
        let stdout = String::from_utf8_lossy(&stripped);
        let target = TargetTriple::from_host().unwrap();
        let fuelup_home = cfg.fuelup_dir();
        let fuelup_home_str = fuelup_home.to_string_lossy();
        let expected_stdout = format!(
            r#"Default host: {target}
fuelup home: {fuelup_home_str}

installed toolchains
--------------------
my_toolchain (default)

active toolchain
----------------
my_toolchain (default)
  forc : not found
    - forc-client
      - forc-deploy : not found
      - forc-run : not found
    - forc-crypto : not found
    - forc-debug : not found
    - forc-doc : not found
    - forc-explore : not found
    - forc-fmt : not found
    - forc-lsp : not found
    - forc-tx : not found
    - forc-wallet : not found
  fuel-core : not found
  fuel-core-keygen : not found
"#
        );

        assert_eq!(stdout, expected_stdout);
    })?;
    Ok(())
}

#[test]
fn fuelup_show_override() -> Result<()> {
    testcfg::setup(FuelupState::LatestWithBetaOverride, &|cfg| {
        let stripped = strip_ansi_escapes::strip(cfg.fuelup(&["show"]).stdout);
        let stdout = String::from_utf8_lossy(&stripped);
        let target = TargetTriple::from_host().unwrap();
        let fuelup_home = cfg.fuelup_dir();
        let fuelup_home_str = fuelup_home.to_string_lossy();
        let expected_stdout = format!(
            r#"Default host: {target}
fuelup home: {fuelup_home_str}

installed toolchains
--------------------
latest-{target} (default)

active toolchain
----------------
beta-1-{target} (override), path: {}
  forc : not found
    - forc-client
      - forc-deploy : not found
      - forc-run : not found
    - forc-crypto : not found
    - forc-debug : not found
    - forc-doc : not found
    - forc-explore : not found
    - forc-fmt : not found
    - forc-lsp : not found
    - forc-tx : not found
    - forc-wallet : not found
  fuel-core : not found
  fuel-core-keygen : not found
"#,
            cfg.home.join(FUEL_TOOLCHAIN_TOML_FILE).display()
        );
        assert_eq!(stdout, expected_stdout);
    })?;
    Ok(())
}

#[test]
fn fuelup_show_latest_then_override() -> Result<()> {
    testcfg::setup(FuelupState::AllInstalled, &|cfg| {
        let mut stripped = strip_ansi_escapes::strip(cfg.fuelup(&["show"]).stdout);
        let mut stdout = String::from_utf8_lossy(&stripped);

        let target = TargetTriple::from_host().unwrap();
        let fuelup_home = cfg.fuelup_dir();
        let fuelup_home_str = fuelup_home.to_string_lossy();
        let expected_stdout = format!(
            r#"Default host: {target}
fuelup home: {fuelup_home_str}

installed toolchains
--------------------
latest-{target} (default)
nightly-{target}
nightly-2022-08-30-{target}

active toolchain
----------------
latest-{target} (default)
  forc : 0.1.0
    - forc-client
      - forc-deploy : 0.1.0
      - forc-run : 0.1.0
    - forc-crypto : 0.1.0
    - forc-debug : 0.1.0
    - forc-doc : 0.1.0
    - forc-explore : 0.1.0
    - forc-fmt : 0.1.0
    - forc-lsp : 0.1.0
    - forc-tx : 0.1.0
    - forc-wallet : 0.1.0
  fuel-core : 0.1.0
  fuel-core-keygen : not found
"#,
        );
        assert_eq!(stdout, expected_stdout);

        let override_path = cfg.home.join(FUEL_TOOLCHAIN_TOML_FILE);
        let ovveride_path_str = override_path.to_string_lossy();
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

        stripped = strip_ansi_escapes::strip(cfg.fuelup(&["show"]).stdout);
        stdout = String::from_utf8_lossy(&stripped);

        let expected_stdout = format!(
            r#"Default host: {target}
fuelup home: {fuelup_home_str}

installed toolchains
--------------------
latest-{target} (default)
nightly-{target}
nightly-2022-08-30-{target} (override)

active toolchain
----------------
nightly-2022-08-30-{target} (override), path: {ovveride_path_str}
  forc : 0.2.0
    - forc-client
      - forc-deploy : 0.2.0
      - forc-run : 0.2.0
    - forc-crypto : 0.2.0
    - forc-debug : 0.2.0
    - forc-doc : 0.2.0
    - forc-explore : 0.2.0
    - forc-fmt : 0.2.0
    - forc-lsp : 0.2.0
    - forc-tx : 0.2.0
    - forc-wallet : 0.2.0
  fuel-core : 0.2.0
  fuel-core-keygen : not found
"#,
        );
        assert_eq!(stdout, expected_stdout);
    })?;
    Ok(())
}
