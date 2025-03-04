use crate::{COMPILER_EXPLORER_REPO, FORK_COMPILER_EXPLORER_REPO, FORK_INFRA_REPO, INFRA_REPO};

use anyhow::{Context, Result};
use std::{path::Path, process::Command};
use tempfile::tempdir;

pub fn clone_fork(
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

pub fn commit_and_push(
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

pub fn create_pull_request(
    repo: &str,
    head: &str,
    version: &str,
    github_token: &str,
) -> Result<()> {
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
