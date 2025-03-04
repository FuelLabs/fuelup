use anyhow::{Context, Result, anyhow};
use std::{env, fs, path::Path, process::Command};
use tempfile::tempdir;

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
    let (infra_temp_dir, infra_path) = clone_fork(FORK_INFRA_REPO, "infra", &branch_name)?;
    let (ce_temp_dir, ce_path) = clone_fork(
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
    commit_and_push(&infra_path, version, "Updated Sway in infra", &branch_name)?;
    commit_and_push(
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
    create_pull_request(
        INFRA_REPO,
        &format!(
            "{}:{}",
            FORK_INFRA_REPO.split('/').next().unwrap(),
            branch_name
        ),
        version,
        &github_token,
    )?;

    create_pull_request(
        COMPILER_EXPLORER_REPO,
        &format!(
            "{}:{}",
            FORK_COMPILER_EXPLORER_REPO.split('/').next().unwrap(),
            branch_name
        ),
        version,
        &github_token,
    )?;

    // print the contents of the edited files
    // println!(
    //     "SWAY YAML:\n{}",
    //     fs::read_to_string(infra_path.join(SWAY_YAML_PATH))?
    // );
    // println!(
    //     "LIBRARIES YAML:\n{}",
    //     fs::read_to_string(infra_path.join(LIBRARIES_YAML_PATH))?
    // );
    // println!(
    //     "AMAZON PROPERTIES:\n{}",
    //     fs::read_to_string(ce_path.join(AMAZON_PROPERTIES_PATH))?
    // );

    // Keep directories alive until the end of the function
    let _ = (infra_temp_dir, ce_temp_dir);

    Ok(())
}

fn clone_fork(
    repo: &str,
    dir_name: &str,
    branch_name: &str,
) -> Result<(tempfile::TempDir, std::path::PathBuf)> {
    let temp_dir = tempdir()?;
    let repo_path = temp_dir.path().join(dir_name);

    // Clone the fork directly
    Command::new("git")
        .args([
            "clone",
            &format!("https://github.com/{}.git", repo),
            repo_path.to_str().unwrap(),
        ])
        .status()
        .with_context(|| format!("Failed to clone {} repo", dir_name))?;

    // Add the upstream repo as "origin"
    let upstream_repo = if repo.starts_with("JoshuaBatty/") {
        format!(
            "https://github.com/compiler-explorer/{}",
            repo.split('/').nth(1).unwrap_or(dir_name)
        )
    } else {
        format!("https://github.com/{}.git", repo)
    };

    Command::new("git")
        .current_dir(&repo_path)
        .args(["remote", "add", "upstream", &upstream_repo])
        .status()
        .with_context(|| format!("Failed to add upstream remote for {}", repo))?;

    // Fetch from upstream
    Command::new("git")
        .current_dir(&repo_path)
        .args(["fetch", "upstream"])
        .status()
        .context("Failed to fetch from upstream")?;

    // Reset to upstream/main
    Command::new("git")
        .current_dir(&repo_path)
        .args(["reset", "--hard", "upstream/main"])
        .status()
        .context("Failed to reset to upstream/main")?;

    // Create a new branch
    Command::new("git")
        .current_dir(&repo_path)
        .args(["checkout", "-b", branch_name])
        .status()
        .with_context(|| format!("Failed to create branch {}", branch_name))?;

    Ok((temp_dir, repo_path))
}

fn commit_and_push(
    repo_path: &Path,
    version: &str,
    message: &str,
    branch_name: &str,
) -> Result<()> {
    // Configure git user
    Command::new("git")
        .current_dir(repo_path)
        .args(["config", "user.name", "Sway Compiler Explorer Bot"])
        .status()
        .context("Failed to set git user name")?;

    Command::new("git")
        .current_dir(repo_path)
        .args([
            "config",
            "user.email",
            "joshpbatty@gmail.com", // Use an appropriate email
        ])
        .status()
        .context("Failed to set git user email")?;

    // Add all changes
    Command::new("git")
        .current_dir(repo_path)
        .args(["add", "."])
        .status()
        .context("Failed to git add")?;

    // Commit
    Command::new("git")
        .current_dir(repo_path)
        .args(["commit", "-m", &format!("{}: {}", message, version)])
        .status()
        .context("Failed to git commit")?;

    // Push with retry
    git_push_with_retry(repo_path, branch_name, 3)?;

    Ok(())
}

fn git_push_with_retry(repo_path: &Path, branch_name: &str, max_retries: usize) -> Result<()> {
    let mut attempt = 0;
    let mut last_error = None;

    while attempt < max_retries {
        println!("Push attempt {} of {}", attempt + 1, max_retries);

        // Try push
        let result = Command::new("git")
            .current_dir(repo_path)
            .args(["push", "-f", "-u", "origin", branch_name])
            .status();

        match result {
            Ok(status) if status.success() => {
                println!("Successfully pushed to branch {}", branch_name);
                return Ok(());
            }
            Ok(status) => {
                println!("Push failed with exit code: {:?}", status.code());
                // Wait before retrying
                std::thread::sleep(std::time::Duration::from_secs(2));
            }
            Err(e) => {
                last_error = Some(e);
                println!("Push error: {:?}. Retrying...", last_error);
                std::thread::sleep(std::time::Duration::from_secs(2));
            }
        }

        attempt += 1;
    }

    // If we get here, all retry attempts failed
    Err(anyhow::anyhow!(
        "Failed to push after {} attempts: {:?}",
        max_retries,
        last_error
    ))
}

fn create_pull_request(repo: &str, head: &str, version: &str, github_token: &str) -> Result<()> {
    // For testing, create PR against the fork itself instead of upstream
    let fork_repo = if repo == INFRA_REPO {
        FORK_INFRA_REPO
    } else if repo == COMPILER_EXPLORER_REPO {
        FORK_COMPILER_EXPLORER_REPO
    } else {
        repo
    };

    // Check if gh CLI is available
    let gh_available = Command::new("gh")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);

    if gh_available {
        // Use GitHub CLI
        println!("Creating PR using GitHub CLI...");
        let output = Command::new("gh")
            .env("GITHUB_TOKEN", github_token)
            .args([
                "pr",
                "create",
                "--repo", fork_repo,
                "--head", head,
                "--base", "main",
                "--title", &format!("Update Sway compiler to version {}", version),
                "--body", &format!("This PR updates the Sway compiler version to {} as per the latest mainnet release.", version),
            ])
            .output()
            .context("Failed to execute gh command")?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            println!("PR creation error: {}", error);
            return Err(anyhow::anyhow!("Failed to create pull request: {}", error));
        }

        let url = String::from_utf8_lossy(&output.stdout);
        println!("Created pull request: {}", url);
    } else {
        // Fallback to using curl with GitHub API
        println!("GitHub CLI not found. Using direct API call...");

        // Parse owner/repo
        let parts: Vec<&str> = fork_repo.split('/').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!("Invalid repository format: {}", fork_repo));
        }
        let owner = parts[0];
        let repo_name = parts[1];

        // Get the branch name without owner prefix
        let branch_name = head.split(':').last().unwrap_or(head);

        // Create JSON payload - no owner prefix in head
        let payload = format!(
            r#"{{"title":"Update Sway compiler to version {}","head":"{}","base":"main","body":"This PR updates the Sway compiler version to {} as per the latest mainnet release."}}"#,
            version, branch_name, version
        );

        println!(
            "API URL: https://api.github.com/repos/{}/{}/pulls",
            owner, repo_name
        );
        println!("PR payload: {}", payload);

        // Use curl with verbose output
        let output = Command::new("curl")
            .args([
                "-v", // Verbose output
                "-X",
                "POST",
                &format!("https://api.github.com/repos/{}/{}/pulls", owner, repo_name),
                "-H",
                "Accept: application/vnd.github.v3+json",
                "-H",
                &format!("Authorization: token {}", github_token),
                "-H",
                "Content-Type: application/json",
                "-d",
                &payload,
            ])
            .output()
            .context("Failed to execute curl command")?;

        let response_body = String::from_utf8_lossy(&output.stdout);
        let error_output = String::from_utf8_lossy(&output.stderr);

        println!("API response: {}", response_body);
        if !error_output.is_empty() {
            println!("Curl stderr: {}", error_output);
        }

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "Failed to create pull request via API. Status code: {}",
                output.status
            ));
        }

        // Check for error message in response
        if response_body.contains("\"message\":") && response_body.contains("\"error\":") {
            return Err(anyhow::anyhow!("GitHub API error: {}", response_body));
        }

        println!("Created pull request via GitHub API");

        // Try to extract PR URL
        if let Some(html_url) = response_body
            .lines()
            .find(|line| line.contains("\"html_url\":") && line.contains("/pull/"))
            .and_then(|line| {
                let parts: Vec<&str> = line.split('"').collect();
                parts
                    .iter()
                    .position(|&s| s == "html_url")
                    .and_then(|pos| parts.get(pos + 2))
                    .map(|s| *s)
            })
        {
            println!("Pull request URL: {}", html_url);
        }
    }

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
