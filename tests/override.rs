use anyhow::Result;
use std::str::FromStr;

pub mod testcfg;
use fuelup::{
    constants::FUEL_TOOLCHAIN_TOML_FILE,
    toolchain_override::{self, OverrideCfg, ToolchainCfg, ToolchainOverride},
};
use testcfg::FuelupState;

#[test]
fn check_correct_forc_deploy_called() -> Result<()> {
    testcfg::setup(FuelupState::AllInstalled, &|cfg| {
        let mut stdout = cfg.forc(&["deploy", "--version"]).stdout;
        assert_eq!(stdout, "forc-deploy 0.1.0\n");
        stdout = cfg.exec("forc-deploy", &["--version"]).stdout;
        assert_eq!(stdout, "forc-deploy 0.1.0\n");

        let toolchain_override = ToolchainOverride {
            cfg: OverrideCfg::new(
                ToolchainCfg {
                    channel: toolchain_override::Channel::from_str("nightly-2022-08-30").unwrap(),
                },
                None,
            ),
            path: cfg.home.join(FUEL_TOOLCHAIN_TOML_FILE),
        };
        testcfg::setup_override_file(toolchain_override).unwrap();

        stdout = cfg.forc(&["deploy", "--version"]).stdout;
        assert_eq!(stdout, "forc-deploy 0.2.0\n");
        stdout = cfg.exec("forc-deploy", &["--version"]).stdout;
        assert_eq!(stdout, "forc-deploy 0.2.0\n");
    })?;

    Ok(())
}
