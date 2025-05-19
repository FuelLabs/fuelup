use anyhow::{Context, Result};
use std::{env, path::Path, process::Command};
use tempfile::tempdir;

pub fn clone_fork(
    repo: &str,
    dir_name: &str,
    branch_name: &str,
    github_token: &str,
) -> Result<(tempfile::TempDir, std::path::PathBuf)> {
    let temp_dir = tempdir()?;
    let repo_path = temp_dir.path().join(dir_name);

    // Clone the fork using the token
    let clone_url = format!(
        "https://x-access-token:{}@github.com/{}.git",
        github_token, repo
    );
    let clone_status = Command::new("git")
        .args([
            "clone",
            &clone_url,
            repo_path.to_str().unwrap(),
        ])
        .status()
        .with_context(|| format!("Failed to clone {} repo using token", dir_name))?;

    if !clone_status.success() {
        return Err(anyhow::anyhow!(
            "Failed to clone {} repo. Exit code: {:?}",
            dir_name,
            clone_status.code()
        ));
    }

    // The cloned fork is 'origin' by default.
    // Add the actual upstream repository.
    let upstream_name = if repo.starts_with("FuelLabs/") {
        repo.split('/').nth(1).unwrap_or(dir_name)
    } else {
        dir_name // Should not happen if FORK_XXX_REPO constants are correct
    };
    let upstream_url = format!("https://github.com/compiler-explorer/{}.git", upstream_name);

    Command::new("git")
        .current_dir(&repo_path)
        .args(["remote", "add", "upstream", &upstream_url])
        .status()
        .with_context(|| format!("Failed to add upstream remote for {}", repo))?;

    Command::new("git")
        .current_dir(&repo_path)
        .args(["fetch", "upstream"])
        .status()
        .context("Failed to fetch from upstream")?;

    let upstream_default_branch = "main";

    let reset_status = Command::new("git")
        .current_dir(&repo_path)
        .args(["reset", "--hard", &format!("upstream/{}", upstream_default_branch)])
        .status()
        .with_context(|| format!("Failed to reset to upstream/{}", upstream_default_branch))?;

    if !reset_status.success() {
        return Err(anyhow::anyhow!(
            "Failed to reset to upstream/{}. Exit code: {:?}",
            upstream_default_branch,
            reset_status.code()
        ));
    }
    
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
    github_token: &str,
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
            "fuel-service-user@users.noreply.github.com",
        ])
        .status()
        .context("Failed to set git user email")?;

    Command::new("git")
        .current_dir(repo_path)
        .args(["add", "."])
        .status()
        .context("Failed to git add")?;

    // Commit, allow it to "fail" if there are no changes (it will exit non-zero)
    let commit_status = Command::new("git")
        .current_dir(repo_path)
        .args(["commit", "-m", &format!("{}: {}", message, version)])
        .status()
        .context("Git commit command failed to execute")?;

    if !commit_status.success() {
        // Check if it failed because there's nothing to commit
        let diff_output = Command::new("git")
            .current_dir(repo_path)
            .args(["diff", "--cached", "--quiet"]) // Checks staged changes
            .status()?;
        
        if diff_output.success() { // success means no diff, so nothing was staged/committed
             println!("No changes to commit for version {}.", version);
        } else {
            // If diff shows changes, or some other commit error
            return Err(anyhow::anyhow!(
                "Failed to git commit. Exit code: {:?}",
                commit_status.code()
            ));
        }
    }

    // Push with retry using the token
    git_push_with_retry(repo_path, branch_name, 3, github_token)?;

    Ok(())
}

pub fn create_pull_request(
    repo: &str,
    head: &str,
    version: &str,
    github_token: &str,
) -> Result<()> {
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
                "--repo", repo,
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
        let parts: Vec<&str> = repo.split('/').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!("Invalid repository format: {}", repo));
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

fn git_push_with_retry(
    repo_path: &Path,
    branch_name: &str,
    max_retries: usize,
    github_token: &str,
) -> Result<()> {
    let mut attempt = 0;
    let mut last_error: Option<anyhow::Error> = None;

    // Get the original origin URL to reformat it with the token
    let origin_url_output = Command::new("git")
        .current_dir(repo_path)
        .args(["config", "--get", "remote.origin.url"])
        .output()
        .context("Failed to get remote.origin.url")?;

    if !origin_url_output.status.success() {
        return Err(anyhow::anyhow!(
            "Failed to get origin URL: {}",
            String::from_utf8_lossy(&origin_url_output.stderr)
        ));
    }
    let origin_url_str = String::from_utf8_lossy(&origin_url_output.stdout).trim().to_string();

    let authenticated_push_url = if origin_url_str.starts_with("https://github.com/") {
        format!(
            "https://x-access-token:{}@{}",
            github_token,
            origin_url_str.trim_start_matches("https://")
        )
    } else if origin_url_str.starts_with(&format!("https://x-access-token:{}@", github_token)) {
        // Already correctly authenticated from clone
        origin_url_str
    }
    else if origin_url_str.starts_with("https://x-access-token:") {
        // Authenticated, but potentially with a different token (e.g. if one token for clone, another for push)
        // This is unlikely given the script's structure but added for robustness.
        let parts: Vec<&str> = origin_url_str.split('@').collect();
        if parts.len() == 2 {
            format!("https://x-access-token:{}@{}", github_token, parts[1])
        } else {
            return Err(anyhow::anyhow!("Unexpected authenticated origin URL format for re-authentication: {}", origin_url_str));
        }
    }
    else {
        return Err(anyhow::anyhow!(
            "Origin URL is not HTTPS, cannot inject token: {}",
            origin_url_str
        ));
    };

    while attempt < max_retries {
        println!("Push attempt {} of {}", attempt + 1, max_retries);

        let result = Command::new("git")
            .current_dir(repo_path)
            .args([
                "push",
                "-f", // Force push is used, ensure this is intended for the target branches
                "-u",
                &authenticated_push_url, // Use the authenticated URL
                &format!("HEAD:{}", branch_name), // Push current HEAD to the remote branch_name
            ])
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
                last_error = Some(e.into());
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
