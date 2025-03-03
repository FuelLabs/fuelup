use anyhow::{Context, Result};
use serde_yaml::{self, Value};
use std::{env, fs, path::Path, process::Command};
use tempfile::tempdir;

const INFRA_REPO: &str = "compiler-explorer/infra";
const COMPILER_EXPLORER_REPO: &str = "compiler-explorer/compiler-explorer";
const SWAY_YAML_PATH: &str = "bin/yaml/sway.yaml";
const LIBRARIES_YAML_PATH: &str = "bin/yaml/libraries.yaml";
const AMAZON_PROPERTIES_PATH: &str = "etc/config/sway.amazon.properties";
const STD_LIB_PATH: &str = "/opt/compiler-explorer/libs/sway/std";

fn main() -> Result<()> {
    let new_version = env::args().nth(1).context("Missing version argument")?;

    let clone_repo = |repo: &str, dir_name: &str| -> Result<_> {
        let temp_dir = tempdir()?;
        let repo_path = temp_dir.path().join(dir_name);
        Command::new("git")
            .args([
                "clone",
                &format!("https://github.com/{}.git", repo),
                repo_path.to_str().unwrap(),
            ])
            .status()
            .with_context(|| format!("Failed to clone {} repo", dir_name))?;

        // Return both temp_dir and repo_path to keep temp_dir alive
        Ok((temp_dir, repo_path))
    };

    // Clone the repositories
    let (_, infra_repo_path) = clone_repo(INFRA_REPO, "infra")?;
    let (_, compiler_explorer_path) = clone_repo(COMPILER_EXPLORER_REPO, "compiler-explorer")?;

    update_amazon_properties(&compiler_explorer_path, &new_version)?;
    update_sway_yaml(&infra_repo_path, &new_version)?;
    update_libraries_yaml(&infra_repo_path, &new_version)?;
    println!("Updated for Sway version {}", new_version);

    // print the contents of the edited files
    println!(
        "SWAY YAML:\n{}",
        fs::read_to_string(infra_repo_path.join(SWAY_YAML_PATH))?
    );
    println!(
        "LIBRARIES YAML:\n{}",
        fs::read_to_string(infra_repo_path.join(LIBRARIES_YAML_PATH))?
    );
    println!(
        "AMAZON PROPERTIES:\n{}",
        fs::read_to_string(compiler_explorer_path.join(AMAZON_PROPERTIES_PATH))?
    );
    Ok(())
}

// Updates the targets section of the sway.yaml file with the given version
fn update_sway_yaml(repo_path: &Path, version: &str) -> Result<()> {
    let path = repo_path.join(SWAY_YAML_PATH);
    let content = fs::read_to_string(&path)?;
    let mut yaml = serde_yaml::from_str::<Value>(&content)?;

    let targets = yaml["compilers"]["sway"]["targets"]
        .as_sequence_mut()
        .context("Invalid YAML structure")?;
    if !targets.iter().any(|v| v.as_str() == Some(version)) {
        targets.push(Value::String(version.to_string()));
        fs::write(&path, serde_yaml::to_string(&yaml)?)?;
        println!("Added {} to sway.yaml", version);
    }
    Ok(())
}

// Updates the targets section of sway in the libraries.yaml file with the given version
fn update_libraries_yaml(repo_path: &Path, version: &str) -> Result<()> {
    let path = repo_path.join(LIBRARIES_YAML_PATH);
    let content = fs::read_to_string(&path)?;
    let mut yaml = serde_yaml::from_str::<Value>(&content)?;

    let targets = yaml["libraries"]["sway"]["std"]["targets"]
        .as_sequence_mut()
        .context("Invalid YAML structure")?;
    if !targets.iter().any(|v| v.as_str() == Some(version)) {
        targets.push(Value::String(version.to_string()));
        fs::write(&path, serde_yaml::to_string(&yaml)?)?;
        println!("Added {} to libraries.yaml", version);
    }
    Ok(())
}

// Adds a new compiler version to the amazon.properties file
fn update_amazon_properties(repo_path: &Path, version: &str) -> Result<()> {
    let path = repo_path.join(AMAZON_PROPERTIES_PATH);
    let mut lines = fs::read_to_string(&path)?
        .lines()
        .map(String::from)
        .collect::<Vec<_>>();

    // Create compiler ID and configure key properties
    let compiler_id = format!("swayv{}", version.replace(".", ""));

    // Update defaultCompiler and compiler list
    for line in &mut lines {
        if line.starts_with("defaultCompiler=") {
            *line = format!("defaultCompiler={}", compiler_id);
        } else if line.starts_with("group.sway.compilers=") && !line.contains(&compiler_id) {
            *line = format!("{}:{}", line, compiler_id);
        }
    }

    // Clean up any problematic entries
    lines.retain(|l| {
        !(l.starts_with(&format!("compiler.{}.std=", compiler_id))
            && !l.ends_with(&format!("/v{}", version)))
            && l != &format!("compiler.{}.semver", compiler_id)
    });

    // Fix existing compiler std lines
    let mut prefixes_to_fix = Vec::new();
    for line in &lines {
        if line.starts_with("compiler.") && line.contains(".semver=") {
            let parts: Vec<&str> = line.split('.').collect();
            if parts.len() >= 3 {
                let prefix = format!("{}.{}", parts[0], parts[1]);
                let ver = line.split('=').nth(1).unwrap_or("").trim();

                let std_key = format!("{}.std", prefix);
                let has_valid_std = lines
                    .iter()
                    .any(|l| l.starts_with(&std_key) && l.ends_with(&format!("/v{}", ver)));

                if !has_valid_std {
                    prefixes_to_fix.push((prefix, ver.to_string()));
                }
            }
        }
    }

    // Check if we need to add the new compiler
    let tools_pos = lines
        .iter()
        .position(|l| l.starts_with("tools="))
        .unwrap_or(lines.len());
    if !lines
        .iter()
        .any(|l| l.starts_with(&format!("compiler.{}.exe", compiler_id)))
    {
        let new_config = format!(
            "compiler.{0}.exe=/opt/compiler-explorer/sway-{1}/forc-binaries/forc\ncompiler.{0}.semver={1}\ncompiler.{0}.name=sway {1}\ncompiler.{0}.std=/opt/compiler-explorer/libs/sway/std/v{1}",
            compiler_id, version
        );
        lines.insert(tools_pos, new_config);
    }

    // Add missing std lines for existing compilers
    for (prefix, ver) in prefixes_to_fix {
        // Remove any incorrect std lines
        lines.retain(|l| {
            !(l.starts_with(&format!("{}.std", prefix)) && !l.ends_with(&format!("/v{}", ver)))
        });

        // Add the std line after name or semver
        if let Some(pos) = lines
            .iter()
            .position(|l| l.starts_with(&format!("{}.name", prefix)))
        {
            lines.insert(pos + 1, format!("{}.std={}/v{}", prefix, STD_LIB_PATH, ver));
        } else if let Some(pos) = lines
            .iter()
            .position(|l| l.starts_with(&format!("{}.semver", prefix)))
        {
            lines.insert(pos + 1, format!("{}.std={}/v{}", prefix, STD_LIB_PATH, ver));
        }
    }

    // Move comment and add empty line
    if let Some(comment_pos) = lines
        .iter()
        .position(|l| l.trim().starts_with("# Basic tools"))
    {
        if let Some(tools_pos) = lines.iter().position(|l| l.starts_with("tools=")) {
            if comment_pos < tools_pos {
                let comment = lines.remove(comment_pos);
                lines.insert(tools_pos - 1, String::new());
                lines.insert(tools_pos, comment);
            }
        }
    }

    fs::write(path, lines.join("\n"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use std::fs;

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
        fs::write(
            &path,
            indoc! {r#"
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
        "#},
        )
        .unwrap();

        update_libraries_yaml(dir.path(), "0.67.0").unwrap();

        // Verify correct section was updated
        let updated = fs::read_to_string(&path).unwrap();
        let yaml: Value = serde_yaml::from_str(&updated).unwrap();
        let targets = yaml["libraries"]["sway"]["std"]["targets"]
            .as_sequence()
            .unwrap();
        assert_eq!(targets.len(), 2);
        assert!(targets.iter().any(|v| v.as_str() == Some("0.66.7")));
        assert!(targets.iter().any(|v| v.as_str() == Some("0.67.0")));
    }

    #[test]
    fn test_update_amazon_properties() {
        let dir = tempdir().unwrap();
        let path = dir.path().join(AMAZON_PROPERTIES_PATH);
        fs::create_dir_all(path.parent().unwrap()).unwrap();

        // Create realistic properties file
        fs::write(
            &path,
            indoc! {r#"
        compilers=&sway
        defaultCompiler=swayv0667
        objdumper=/opt/compiler-explorer/gcc-14.2.0/bin/objdump
        group.sway.compilers=swayv0667
        group.sway.compilerType=sway-compiler
        group.sway.isSemVer=true
        group.sway.supportsIrView=true
        group.sway.irArg=--ir final
        group.sway.supportsBinary=true
        group.sway.supportsAsm=true
        group.sway.asmArg=--asm all
        compiler.swayv0667.exe=/opt/compiler-explorer/sway-0.66.7/forc-binaries/forc
        compiler.swayv0667.semver=0.66.7
        compiler.swayv0667.name=sway 0.66.7
        # Basic tools that might be useful
        tools=
        "#},
        )
        .unwrap();

        // Test adding a new version
        update_amazon_properties(dir.path(), "0.67.0").unwrap();

        let content = fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = content.lines().collect();

        // Check individual required elements
        assert!(lines.contains(&"defaultCompiler=swayv0670"));
        assert!(lines.contains(&"group.sway.compilers=swayv0667:swayv0670"));
        assert!(lines.contains(
            &"compiler.swayv0670.exe=/opt/compiler-explorer/sway-0.67.0/forc-binaries/forc"
        ));
        assert!(lines.contains(&"compiler.swayv0670.semver=0.67.0"));
        assert!(lines.contains(&"compiler.swayv0670.name=sway 0.67.0"));
        assert!(
            lines.contains(&"compiler.swayv0670.std=/opt/compiler-explorer/libs/sway/std/v0.67.0")
        );
    }
}
