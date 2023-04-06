use anyhow::Result;
use std::str::FromStr;

pub mod testcfg;
use fuelup::{
    constants::FUEL_TOOLCHAIN_TOML_FILE,
    toolchain_override::{self, OverrideCfg, ToolchainCfg, ToolchainOverride},
};
use testcfg::FuelupState;

#[test]
fn check_correct_forc_plugin_called() -> Result<()> {
    // We execute both 'forc wallet' and 'forc-wallet' in this test
    // to ensure both work as intended.
    //
    // The test environment is set up similarly to a real fuelup environment,
    // complete with links.
    testcfg::setup(FuelupState::AllInstalled, &|cfg| {
        let mut stdout = cfg.forc(&["wallet", "--version"]).stdout;
        assert_eq!(stdout, "forc-wallet 0.1.0\n");
        stdout = cfg.exec("forc-wallet", &["--version"]).stdout;
        assert_eq!(stdout, "forc-wallet 0.1.0\n");

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

        stdout = cfg.forc(&["wallet", "--version"]).stdout;
        assert_eq!(stdout, "forc-wallet 0.2.0\n");
        stdout = cfg.exec("forc-wallet", &["--version"]).stdout;
        assert_eq!(stdout, "forc-wallet 0.2.0\n");
    })?;

    Ok(())
}
