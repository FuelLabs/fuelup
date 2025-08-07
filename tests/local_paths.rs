pub mod testcfg;

use anyhow::Result;
use fuelup::{
    constants::FUEL_TOOLCHAIN_TOML_FILE,
    toolchain_override::{ComponentSpec, OverrideCfg, ToolchainCfg, ToolchainOverride},
};
use std::{
    collections::HashMap,
    fs::{File, Permissions},
    path::PathBuf,
    str::FromStr,
};
use testcfg::FuelupState;

#[test]
fn test_local_path_integration() -> Result<()> {
    testcfg::setup(FuelupState::AllInstalled, &|cfg| {
        // Create a mock local forc binary
        let local_forc_path = cfg.home.join("local_forc");
        File::create(&local_forc_path).unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = Permissions::from_mode(0o755);
            std::fs::set_permissions(&local_forc_path, perms).unwrap();
        }

        // Create fuel-toolchain.toml with mixed version/path configuration
        let mut components = HashMap::new();
        components.insert(
            "forc".to_string(),
            ComponentSpec::Path(local_forc_path.clone()),
        );
        components.insert(
            "fuel-core".to_string(),
            ComponentSpec::Version(semver::Version::new(0, 41, 7)),
        );

        let toolchain_override = ToolchainOverride {
            cfg: OverrideCfg::new(
                ToolchainCfg {
                    channel: fuelup::toolchain_override::Channel::from_str("testnet").unwrap(),
                },
                Some(components),
            ),
            path: cfg.home.join(FUEL_TOOLCHAIN_TOML_FILE),
        };

        testcfg::setup_override_file(toolchain_override.clone()).unwrap();

        // Validate that local components can be validated successfully
        #[cfg(unix)]
        {
            let result = toolchain_override.validate_local_components();
            assert!(
                result.is_ok(),
                "Local component validation should succeed: {:?}",
                result.err()
            );
        }

        // Test that we can retrieve component specs correctly
        let forc_spec = toolchain_override.get_component_spec("forc").unwrap();
        assert!(forc_spec.is_path());

        let fuel_core_spec = toolchain_override.get_component_spec("fuel-core").unwrap();
        assert!(fuel_core_spec.is_version());

        // Test path resolution
        let resolved_path = toolchain_override.get_component_path("forc").unwrap();
        assert_eq!(resolved_path, local_forc_path);

        // Test that version retrieval works correctly
        assert!(toolchain_override.get_component_version("forc").is_none());
        assert!(toolchain_override
            .get_component_version("fuel-core")
            .is_some());
    })?;
    Ok(())
}

#[test]
fn test_relative_path_resolution() -> Result<()> {
    testcfg::setup(FuelupState::AllInstalled, &|cfg| {
        // Create a local binary in a subdirectory
        let bin_dir = cfg.home.join("bin");
        std::fs::create_dir(&bin_dir).unwrap();
        let local_forc_path = bin_dir.join("forc");
        File::create(&local_forc_path).unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = Permissions::from_mode(0o755);
            std::fs::set_permissions(&local_forc_path, perms).unwrap();
        }

        // Create fuel-toolchain.toml with relative path
        let mut components = HashMap::new();
        components.insert(
            "forc".to_string(),
            ComponentSpec::Path(PathBuf::from("bin/forc")),
        );

        let toolchain_override = ToolchainOverride {
            cfg: OverrideCfg::new(
                ToolchainCfg {
                    channel: fuelup::toolchain_override::Channel::from_str("testnet").unwrap(),
                },
                Some(components),
            ),
            path: cfg.home.join(FUEL_TOOLCHAIN_TOML_FILE),
        };

        // Test that relative path resolves correctly
        let resolved_path = toolchain_override.get_component_path("forc").unwrap();
        assert_eq!(resolved_path, local_forc_path);

        // Test base directory calculation
        assert_eq!(toolchain_override.base_dir(), cfg.home);
    })?;
    Ok(())
}

#[test]
fn test_toml_serialization_roundtrip() -> Result<()> {
    testcfg::setup(FuelupState::AllInstalled, &|_cfg| {
        // Create fuel-toolchain.toml with mixed configuration
        let mut components = HashMap::new();
        components.insert(
            "forc".to_string(),
            ComponentSpec::Path(PathBuf::from("/usr/local/bin/forc")),
        );
        components.insert(
            "fuel-core".to_string(),
            ComponentSpec::Version(semver::Version::new(0, 41, 7)),
        );

        let original_cfg = OverrideCfg::new(
            ToolchainCfg {
                channel: fuelup::toolchain_override::Channel::from_str("testnet").unwrap(),
            },
            Some(components),
        );

        // Serialize to TOML
        let toml_str = original_cfg.to_string_pretty().unwrap();

        // Verify the TOML contains expected content
        assert!(toml_str.contains("forc = \"/usr/local/bin/forc\""));
        assert!(toml_str.contains("fuel-core = \"0.41.7\""));
        assert!(toml_str.contains("channel = \"testnet\""));

        // Parse back from TOML
        let parsed_cfg = OverrideCfg::from_toml(&toml_str).unwrap();
        let parsed_components = parsed_cfg.components.unwrap();

        // Verify components were parsed correctly
        assert!(parsed_components.get("forc").unwrap().is_path());
        assert!(parsed_components.get("fuel-core").unwrap().is_version());
    })?;
    Ok(())
}
