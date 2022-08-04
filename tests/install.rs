use anyhow::Result;
use std::{env, os::unix::prelude::CommandExt};

mod workdir;

#[test]
fn smoke_test() -> Result<()> {
    workdir::setup(&|cfg| {
        let expected_version = format!("fuelup {}\n", clap::crate_version!());

        let output = cfg.exec_cmd(&["--version"]);
        let stdout = String::from_utf8_lossy(&output.stdout);

        assert_eq!(stdout, expected_version);
    })?;

    Ok(())
}

#[test]
fn fuelup_toolchain_install_latest() -> Result<()> {
    workdir::setup(&|cfg| {
        cfg.exec_cmd(&["toolchain", "install", "latest"]);

        let expected_bins = ["forc", "forc-explore", "fuel-core", "forc-lsp", "forc-fmt"];

        for entry in cfg.toolchains_dir().read_dir().expect("Could not read dir") {
            let toolchain_dir = entry.unwrap();
            assert_eq!("latest-x86_64-apple-darwin", toolchain_dir.file_name());
            assert!(toolchain_dir.file_type().unwrap().is_dir());

            let downloaded_bins: Vec<String> = toolchain_dir
                .path()
                .join("bin")
                .read_dir()
                .expect("Could not read toolchain bin dir")
                .into_iter()
                .map(|b| b.unwrap().file_name().to_string_lossy().to_string())
                .collect();

            assert_eq!(downloaded_bins, expected_bins);
        }
    })?;

    Ok(())
}
