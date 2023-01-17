use anyhow::Result;

mod testcfg;
use testcfg::FuelupState;

#[test]
fn fuelup_completions() -> Result<()> {
    testcfg::setup(FuelupState::LatestToolchainInstalled, &|cfg| {
        let shells = ["zsh", "bash", "fish", "powershell", "elvish"];
        for shell in shells {
            let output = cfg.fuelup(&["completions", "--shell", shell]);

            assert!(output.status.success());
        }
    })?;

    Ok(())
}
