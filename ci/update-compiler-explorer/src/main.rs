use std::{env, fs, path::{Path, PathBuf}, process::Command};
use tempfile::tempdir;
use serde_yaml::{self, Value};

const INFRA_REPO: &str = "compiler-explorer/infra";
const SWAY_YAML_PATH: &str = "bin/yaml/sway.yaml";
const LIBRARIES_YAML_PATH: &str = "bin/yaml/libraries.yaml";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let new_version = env::args().nth(1).ok_or("Missing version argument")?;
    let infra_repo_path = clone_infra_repo()?;
    update_sway_yaml(&infra_repo_path, &new_version)?;
    update_libraries_yaml(&infra_repo_path, &new_version)?;

    println!("Updated for Sway version {}", new_version);

    // print the contents of the 2 yaml files
    println!("SWAY YAML:\n{}", fs::read_to_string(infra_repo_path.join(SWAY_YAML_PATH))?);
    println!("LIBRARIES YAML:\n{}", fs::read_to_string(infra_repo_path.join(LIBRARIES_YAML_PATH))?);
    Ok(())
}

// Clones the infra repo to a temporary directory
fn clone_infra_repo() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let repo_path = temp_dir.path().join("infra");
    Command::new("git")
        .args(["clone", &format!("https://github.com/{}.git", INFRA_REPO), repo_path.to_str().unwrap()])
        .status()?;
    Ok(repo_path)
}

// Updates the targets section of the sway.yaml file with the given version
fn update_sway_yaml(repo_path: &Path, version: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = repo_path.join(SWAY_YAML_PATH);
    let content = fs::read_to_string(&path)?;
    let mut yaml = serde_yaml::from_str::<Value>(&content)?;
    
    let targets = yaml["compilers"]["sway"]["targets"].as_sequence_mut().ok_or("Invalid YAML structure")?;
    if !targets.iter().any(|v| v.as_str() == Some(version)) {
        targets.push(Value::String(version.to_string()));
        fs::write(&path, serde_yaml::to_string(&yaml)?)?;
        println!("Added {} to sway.yaml", version);
    }
    Ok(())
}

// Updates the targets section of sway in the libraries.yaml file with the given version
fn update_libraries_yaml(repo_path: &Path, version: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = repo_path.join(LIBRARIES_YAML_PATH);
    let content = fs::read_to_string(&path)?;
    let mut yaml = serde_yaml::from_str::<Value>(&content)?;
    
    let targets = yaml["libraries"]["sway"]["std"]["targets"].as_sequence_mut().ok_or("Invalid YAML structure")?;
    if !targets.iter().any(|v| v.as_str() == Some(version)) {
        targets.push(Value::String(version.to_string()));
        fs::write(&path, serde_yaml::to_string(&yaml)?)?;
        println!("Added {} to libraries.yaml", version);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use indoc::indoc;

    #[test]
    fn test_update_sway_yaml() {
        let dir = tempdir().unwrap();
        let path = dir.path().join(SWAY_YAML_PATH);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        
        // Create realistic test YAML
        fs::write(&path, indoc! {r#"
        compilers:
            sway:
                type: tarballs
                compression: gz
                url: https://github.com/FuelLabs/sway/releases/download/v{{name}}/forc-binaries-linux_amd64.tar.gz
                check_exe: forc-binaries/forc --version
                dir: sway-{{name}}
                strip_components: 1
                create_untar_dir: true
                targets:
                    - 0.66.7
        "#}).unwrap();
        
        update_sway_yaml(dir.path(), "0.67.0").unwrap();
        
        // Verify 0.67.0 was added without removing 0.66.7
        let updated = fs::read_to_string(&path).unwrap();
        let yaml: Value = serde_yaml::from_str(&updated).unwrap();
        let targets = yaml["compilers"]["sway"]["targets"].as_sequence().unwrap();
        assert_eq!(targets.len(), 2);
        assert!(targets.iter().any(|v| v.as_str() == Some("0.66.7")));
        assert!(targets.iter().any(|v| v.as_str() == Some("0.67.0")));
    }

    #[test]
    fn test_update_libraries_yaml() {
        let dir = tempdir().unwrap();
        let path = dir.path().join(LIBRARIES_YAML_PATH);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        
        // Create realistic test YAML
        fs::write(&path, indoc!{r#"
        libraries:
            other_lib:
                type: example 
            sway:
                std:
                    repo: fuellabs/sway
                    check_file: README.md
                    method: clone_branch
                    build_type: none
                    target_prefix: v
                    targets:
                        - 0.66.7
                    type: github
            another_lib:
                type: example
        "#}).unwrap();
        
        update_libraries_yaml(dir.path(), "0.67.0").unwrap();
        
        // Verify correct section was updated
        let updated = fs::read_to_string(&path).unwrap();
        let yaml: Value = serde_yaml::from_str(&updated).unwrap();
        let targets = yaml["libraries"]["sway"]["std"]["targets"].as_sequence().unwrap();
        assert_eq!(targets.len(), 2);
        assert!(targets.iter().any(|v| v.as_str() == Some("0.66.7")));
        assert!(targets.iter().any(|v| v.as_str() == Some("0.67.0")));
    }
}