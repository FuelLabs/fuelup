use anyhow::{Context, Result};
use std::{path::Path, process::Command};
use tempfile::tempdir;

pub fn clone_fork(
    repo: &str,
    dir_name: &str,
    branch_name: &str,
    github_token: &str,
) -> Result<(tempfile::TempDir, std::path::PathBuf)> {
    let temp_dir = tempdir()?;
    let repo_path = temp_dir.path().join(dir_name);

    let clone_url = format!(
        "https://x-access-token:{}@github.com/{}.git",
        github_token, repo
    );
    let clone_status = Command::new("git")
        .args([
            "clone",
            "--verbose",
            &clone_url,
            repo_path.to_str().unwrap(),
        ])
        .status()
        .with_context(|| format!("Failed to execute git clone for {} repo", dir_name))?;

    if !clone_status.success() {
        return Err(anyhow::anyhow!(
            "Failed to clone {} repo ('{}') with token. Exit code: {:?}. Ensure GITHUB_TOKEN has correct permissions (e.g., contents: read/write) for the repository.",
            dir_name,
            repo,
            clone_status.code(),
        ));
    }

    // Map the repo to the upstream name
    let upstream_name = if repo.starts_with("FuelLabs/") {
        match repo {
            "FuelLabs/compiler-explorer-infra" => "infra",
            "FuelLabs/compiler-explorer" => "compiler-explorer",
            _ => unreachable!("Unexpected repo: {}", repo),
        }
    } else {
        dir_name
    };
    let upstream_url = format!("https://github.com/compiler-explorer/{}.git", upstream_name);

    let remote_add_status = Command::new("git")
        .current_dir(&repo_path)
        .args(["remote", "add", "upstream", &upstream_url])
        .status()
        .with_context(|| format!("Failed to add upstream remote for {}", repo))?;
    if !remote_add_status.success() {
        return Err(anyhow::anyhow!(
            "Failed to add upstream remote ('{}') for {}. Exit code: {:?}",
            upstream_url,
            repo,
            remote_add_status.code()
        ));
    }

    let fetch_status = Command::new("git")
        .current_dir(&repo_path)
        .args(["-c", "credential.helper=", "fetch", "--verbose", "upstream"])
        .status()
        .context("Failed to execute git fetch upstream")?;

    if !fetch_status.success() {
        return Err(anyhow::anyhow!(
            "Failed to fetch from upstream remote {}. Exit code: {:?}. This might indicate an issue fetching a public repository or an unexpected authentication prompt.",
            upstream_url,
            fetch_status.code()
        ));
    }

    let upstream_default_branch = "main";

    let reset_status = Command::new("git")
        .current_dir(&repo_path)
        .args([
            "reset",
            "--hard",
            &format!("upstream/{}", upstream_default_branch),
        ])
        .status()
        .with_context(|| format!("Failed to reset to upstream/{}", upstream_default_branch))?;

    if !reset_status.success() {
        return Err(anyhow::anyhow!(
            "Failed to reset to upstream/{}. Exit code: {:?}",
            upstream_default_branch,
            reset_status.code()
        ));
    }

    let checkout_status = Command::new("git")
        .current_dir(&repo_path)
        .args(["checkout", "-b", branch_name])
        .status()
        .with_context(|| format!("Failed to execute git checkout -b {}", branch_name))?;

    if !checkout_status.success() {
        return Err(anyhow::anyhow!(
            "Failed to create branch {}. Exit code: {:?}",
            branch_name,
            checkout_status.code()
        ));
    }

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
    configure_git_user(repo_path)?;

    // Add everything except .github/workflows
    let add_status = Command::new("git")
        .current_dir(repo_path)
        .args(["add", ".", ":(exclude).github/workflows/*"])
        .status()
        .context("Failed to execute git add with exclusions")?;

    if !add_status.success() {
        return Err(anyhow::anyhow!(
            "Failed to git add changes (excluding workflows). Exit code: {:?}",
            add_status.code()
        ));
    }

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

        if diff_output.success() {
            // success means no diff, so nothing was staged/committed
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

fn configure_git_user(repo_path: &Path) -> Result<()> {
    let config_user_name_status = Command::new("git")
        .current_dir(repo_path)
        .args(["config", "user.name", "Sway Compiler Explorer Bot"])
        .status()
        .context("Failed to execute git config user.name")?;

    if !config_user_name_status.success() {
        return Err(anyhow::anyhow!(
            "Failed to set git user.name. Exit code: {:?}",
            config_user_name_status.code()
        ));
    }

    let config_user_email_status = Command::new("git")
        .current_dir(repo_path)
        .args([
            "config",
            "user.email",
            "fuel-service-user@users.noreply.github.com",
        ])
        .status()
        .context("Failed to execute git config user.email")?;

    if !config_user_email_status.success() {
        return Err(anyhow::anyhow!(
            "Failed to set git user.email. Exit code: {:?}",
            config_user_email_status.code()
        ));
    }

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
            let error_msg = String::from_utf8_lossy(&output.stderr);
            println!("PR creation error (gh): {}", error_msg);
            return Err(anyhow::anyhow!(
                "Failed to create pull request using gh: {}. Exit code: {:?}",
                error_msg,
                output.status.code()
            ));
        }

        let url = String::from_utf8_lossy(&output.stdout);
        println!("Created pull request: {}", url.trim());
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

        // Removed API URL and payload print here, curl -v will show it.

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

        if !output.status.success() {
            println!("Curl command failed. HTTP Status: {}", output.status);
            println!("Curl response body: {}", response_body);
            println!("Curl stderr (verbose logs): {}", error_output);
            return Err(anyhow::anyhow!(
                "Failed to create pull request via API (curl). Status code: {}. See CI logs for curl's verbose output.",
                output.status
            ));
        }

        // Check for error message in response
        if response_body.contains("\"message\":")
            && (response_body.contains("\"errors\":")
                || response_body.to_lowercase().contains("problem"))
            && !response_body.contains("\"html_url\":")
        {
            println!(
                "GitHub API reported an error in response body: {}",
                response_body
            );
            return Err(anyhow::anyhow!(
                "GitHub API call succeeded with HTTP {} but reported an error in the response body: {}",
                output.status,
                response_body
            ));
        }

        println!(
            "Successfully created pull request via GitHub API (HTTP {}).",
            output.status
        );

        // Try to extract PR URL
        if let Some(html_url) = response_body
            .lines()
            .find(|line| line.contains("\"html_url\":") && line.contains("/pull/"))
            .and_then(|line| {
                let parts: Vec<&str> = line.split('"').collect();
                parts
                    .get(parts.iter().position(|&s| s == "html_url")? + 2)
                    .copied()
            })
        {
            println!("Pull request URL: {}", html_url);
        } else {
            println!(
                "Could not reliably extract PR URL from API response. Full response: {}",
                response_body
            );
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
            "Failed to get origin URL: {}. Exit code: {:?}",
            String::from_utf8_lossy(&origin_url_output.stderr),
            origin_url_output.status.code()
        ));
    }
    let origin_url_str = String::from_utf8_lossy(&origin_url_output.stdout)
        .trim()
        .to_string();

    let authenticated_push_url = if origin_url_str.starts_with("https://github.com/") {
        format!(
            "https://x-access-token:{}@{}",
            github_token,
            origin_url_str.trim_start_matches("https://")
        )
    } else if origin_url_str.starts_with(&format!("https://x-access-token:{}@", github_token)) {
        // Already correctly authenticated from clone
        origin_url_str
    } else if origin_url_str.starts_with("https://x-access-token:") {
        // Authenticated, but potentially with a different token (e.g. if one token for clone, another for push)
        // This is unlikely given the script's structure but added for robustness.
        let parts: Vec<&str> = origin_url_str.split('@').collect();
        if parts.len() == 2 {
            format!("https://x-access-token:{}@{}", github_token, parts[1])
        } else {
            return Err(anyhow::anyhow!(
                "Unexpected authenticated origin URL format for re-authentication: {}",
                origin_url_str
            ));
        }
    } else {
        return Err(anyhow::anyhow!(
            "Origin URL is not HTTPS, cannot inject token: {}",
            origin_url_str
        ));
    };

    while attempt < max_retries {
        println!("Push attempt {} of {}", attempt + 1, max_retries);

        let push_status_result = Command::new("git")
            .current_dir(repo_path)
            .args([
                "push",
                "--verbose",
                "-f",
                "-u",
                &authenticated_push_url,
                &format!("HEAD:{}", branch_name),
            ])
            .status()
            .context("Failed to execute git push command");

        match push_status_result {
            Ok(status) if status.success() => {
                println!("Successfully pushed to branch {}", branch_name);
                return Ok(());
            }
            Ok(status) => {
                println!("Push failed with exit code: {:?}", status.code());
                last_error = Some(anyhow::anyhow!(
                    "Push attempt {} failed with exit code: {:?}",
                    attempt + 1,
                    status.code()
                ));
                std::thread::sleep(std::time::Duration::from_secs(2));
            }
            Err(e) => {
                last_error = Some(e);
                println!(
                    "Push command execution error: {:?}. Retrying...",
                    last_error
                );
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
