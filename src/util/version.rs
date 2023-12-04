use anyhow::{bail, Result};
use semver::Version;
use std::path::Path;

pub fn exec_version(component_executable: &Path) -> Result<Version> {
    match std::process::Command::new(component_executable)
        .arg("--version")
        .output()
    {
        Ok(o) => {
            let output = String::from_utf8_lossy(&o.stdout).into_owned();
            match output.split_whitespace().last() {
                Some(v) => {
                    let version = Version::parse(v)?;
                    Ok(version)
                }
                None => {
                    bail!("Error getting version string");
                }
            }
        }
        Err(e) => {
            if component_executable.exists() {
                bail!("execution error - {}", e);
            } else {
                bail!("not found");
            }
        }
    }
}