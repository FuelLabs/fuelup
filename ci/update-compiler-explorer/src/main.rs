mod git;

use anyhow::{Context, Result, anyhow};
use std::{env, fs};

const INFRA_REPO: &str = "compiler-explorer/infra";
const COMPILER_EXPLORER_REPO: &str = "compiler-explorer/compiler-explorer";
const SWAY_YAML_PATH: &str = "bin/yaml/sway.yaml";
const LIBRARIES_YAML_PATH: &str = "bin/yaml/libraries.yaml";
const AMAZON_PROPERTIES_PATH: &str = "etc/config/sway.amazon.properties";
const STD_LIB_PATH: &str = "/opt/compiler-explorer/libs/sway/std";

// Forks to use for PRs
const FORK_INFRA_REPO: &str = "JoshuaBatty/infra";
const FORK_COMPILER_EXPLORER_REPO: &str = "JoshuaBatty/compiler-explorer";

// For GitHub API operations
const GITHUB_TOKEN: &str = "GITHUB_TOKEN"; // Environment variable name

fn main() -> Result<()> {
    // Check if we're running in manual mode with a specific version
    let manual_version = env::args().nth(1);

    // Get the Forc version
    let forc_version = match manual_version {
        Some(version) => version,
        None => {
            // Automatically extract version from fuelup's mainnet channel file
            extract_forc_version_from_fuelup()?
        }
    };

    println!("Using Sway/Forc version: {}", forc_version);

    // Clone and update the repositories
    update_compiler_explorer(&forc_version)?;

    Ok(())
}

fn update_compiler_explorer(version: &str) -> Result<()> {
    // Create a unique branch name based on the version
    let branch_name = format!("update-sway-{}", version);

    // Clone the forks
    let (infra_temp_dir, infra_path) = git::clone_fork(FORK_INFRA_REPO, "infra", &branch_name)?;
    let (ce_temp_dir, ce_path) = git::clone_fork(
        FORK_COMPILER_EXPLORER_REPO,
        "compiler-explorer",
        &branch_name,
    )?;

    // Make updates
    let content = fs::read_to_string(&ce_path.join(AMAZON_PROPERTIES_PATH))?;
    update_amazon_properties(&content, version)?;

    let content = fs::read_to_string(&infra_path.join(SWAY_YAML_PATH))?;
    update_sway_yaml(&content, version)?;

    let content = fs::read_to_string(&infra_path.join(LIBRARIES_YAML_PATH))?;
    update_libraries_yaml(&content, version)?;

    println!("Updated for Sway version {}", version);

    // Commit and push changes
    git::commit_and_push(&infra_path, version, "Updated Sway in infra", &branch_name)?;
    git::commit_and_push(
        &ce_path,
        version,
        "Updated Sway in compiler-explorer",
        &branch_name,
    )?;

    println!("Committed and pushed changes to forks");

    // Get the GitHub token from environment
    let github_token = env::var(GITHUB_TOKEN).context(
        "GitHub token not found in environment. Set the GITHUB_TOKEN environment variable.",
    )?;

    // Create pull requests
    git::create_pull_request(
        INFRA_REPO,
        &format!(
            "{}:{}",
            FORK_INFRA_REPO.split('/').next().unwrap(),
            branch_name
        ),
        version,
        &github_token,
    )?;

    git::create_pull_request(
        COMPILER_EXPLORER_REPO,
        &format!(
            "{}:{}",
            FORK_COMPILER_EXPLORER_REPO.split('/').next().unwrap(),
            branch_name
        ),
        version,
        &github_token,
    )?;

    // Keep directories alive until the end of the function
    let _ = (infra_temp_dir, ce_temp_dir);

    Ok(())
}

/// Updates the targets section of the sway.yaml file with the given version
/// Finds the last target and appends the new version after it
fn update_sway_yaml(content: &str, version: &str) -> Result<String> {
    let mut lines: Vec<String> = content.lines().map(String::from).collect();

    // Find the first target line
    let first_target_idx = lines
        .iter()
        .position(|l| l.trim().starts_with("- "))
        .ok_or_else(|| anyhow!("Could not find any target lines"))?;

    // Find the last target line
    let last_target_idx = lines
        .iter()
        .enumerate()
        .skip(first_target_idx)
        .take_while(|(_, l)| l.trim().starts_with("- "))
        .last()
        .map(|(i, _)| i)
        .unwrap_or(first_target_idx);

    // Extract indentation from the first target
    let indent = lines[first_target_idx]
        .chars()
        .take_while(|c| c.is_whitespace())
        .collect::<String>();

    // Check if the version already exists
    let version_exists = lines
        .iter()
        .skip(first_target_idx)
        .take(last_target_idx - first_target_idx + 1)
        .any(|l| l.trim() == format!("- {}", version));

    if version_exists {
        // Return the original content unchanged
        return Ok(content.to_string());
    }

    // Insert new version at the end of the targets list
    let new_line = format!("{}- {}", indent, version);
    lines.insert(last_target_idx + 1, new_line);

    // Preserve trailing newline if it existed in the original
    let result = if content.ends_with('\n') {
        lines.join("\n") + "\n"
    } else {
        lines.join("\n")
    };
    Ok(result)
}

/// Updates the targets section of the sway std library in libraries.yaml with the given version
fn update_libraries_yaml(content: &str, version: &str) -> Result<String> {
    let mut lines: Vec<String> = content.lines().map(String::from).collect();

    // Find the "sway" section
    let sway_idx = lines
        .iter()
        .position(|l| l.trim() == "sway:")
        .ok_or_else(|| anyhow!("Could not find sway section"))?;

    // Find the targets section within the sway std section
    let mut in_sway_section = false;
    let mut targets_idx = 0;

    for (i, line) in lines.iter().enumerate().skip(sway_idx) {
        if line.trim() == "std:" && !in_sway_section {
            in_sway_section = true;
            continue;
        }
        if in_sway_section && line.trim() == "targets:" {
            targets_idx = i;
            break;
        }
    }

    if targets_idx == 0 {
        return Err(anyhow!(
            "Could not find targets section in sway std library"
        ));
    }

    // Get the indentation from the first target line
    let first_target_idx = targets_idx + 1;
    if first_target_idx >= lines.len() {
        return Err(anyhow!("Targets section has no elements"));
    }

    let indent = lines[first_target_idx]
        .chars()
        .take_while(|c| c.is_whitespace())
        .collect::<String>();

    // Find the last target line
    let last_target_idx = lines
        .iter()
        .enumerate()
        .skip(first_target_idx)
        .take_while(|(_, l)| l.trim().starts_with('-'))
        .last()
        .map(|(i, _)| i)
        .unwrap_or(first_target_idx);

    // Check if the version already exists
    let version_exists = lines
        .iter()
        .skip(first_target_idx)
        .take(last_target_idx - first_target_idx + 1)
        .any(|l| l.contains(version));

    if version_exists {
        // Return the original content unchanged
        return Ok(content.to_string());
    }

    // Insert new version with the same indentation at the end of the targets list
    let new_line = format!("{}- {}", indent, version);
    lines.insert(last_target_idx + 1, new_line);

    Ok(lines.join("\n"))
}

/// Adds a new compiler version to the amazon.properties file
fn update_amazon_properties(content: &str, version: &str) -> Result<String> {
    let mut lines = content.lines().map(String::from).collect::<Vec<_>>();

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

    Ok(lines.join("\n"))
}

// Extracts the Forc version from the fuelup channel-fuel-mainnet.toml file
fn extract_forc_version_from_fuelup() -> Result<String> {
    // Direct URL to the raw file
    let file_url =
        "https://raw.githubusercontent.com/FuelLabs/fuelup/gh-pages/channel-fuel-mainnet.toml";

    // Fetch the file content directly
    let response =
        reqwest::blocking::get(file_url).context("Failed to fetch channel-fuel-mainnet.toml")?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Failed to fetch file: HTTP {}",
            response.status()
        ));
    }

    let channel_content = response.text().context("Failed to read response body")?;

    // Parse TOML
    let channel_toml: toml::Table =
        toml::from_str(&channel_content).context("Failed to parse channel-fuel-mainnet.toml")?;

    // Extract Forc version
    let forc_version = channel_toml
        .get("pkg")
        .and_then(|pkg| pkg.get("forc"))
        .and_then(|forc| forc.get("version"))
        .and_then(|version| version.as_str())
        .context("Failed to extract Forc version from TOML")?;

    Ok(forc_version.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use serde_yaml::{self, Value};
    use std::fs;
    use tempfile::tempdir;

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

        // Read the content, update it, and write it back
        let content = fs::read_to_string(&path).unwrap();
        let updated_content = update_sway_yaml(&content, "0.67.0").unwrap();
        fs::write(&path, updated_content).unwrap();

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

        // Read content, update it and write it back
        let content = fs::read_to_string(&path).unwrap();
        let updated_content = update_libraries_yaml(&content, "0.67.0").unwrap();
        fs::write(&path, updated_content).unwrap();

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
        let original_content = indoc! {r#"
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
        "#};

        fs::write(&path, original_content).unwrap();

        // Read the content, update it, and write it back
        let content = fs::read_to_string(&path).unwrap();
        let updated_content = update_amazon_properties(&content, "0.67.0").unwrap();
        fs::write(&path, updated_content).unwrap();

        // Read the updated file
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
