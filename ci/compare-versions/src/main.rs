//! # `compare-versions`
//!
//! This crate queries the GitHub API for forc and fuel-core versions newer than the latest
//! published versions within channel-fuel-latest.toml and collects these versions along with
//! the last published version(s). Then it formats these versions into strings and
//! prints them out to be used as a JSON input into `test-toolchain-compatibility.yml`.
//!
//! If only one of `forc` or `fuel-core` has a new release, only the last published version of the
//! other binary is collected. In this scenario we only need to run tests for that one release vs.
//! the already published binary.
//!
//! If both have new releases, then the last published versions of both binaries are collected.
//! Reason is that it isn't sufficient to test only the newly released versions, since they may
//! both fail. We have to also test the new releases against the last published version sets that
//! we know are compatible, so we can update the channel if necessary.

use anyhow::{bail, Result};
use component::Components;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::{io::Read, os::unix::process::CommandExt, process::Command, str::FromStr};
use toml_edit::Document;

const GITHUB_API_REPOS_BASE_URL: &str = "https://api.github.com/repos/FuelLabs/";
const ACTIONS_RUNS: &str = "actions/runs";
const SWAY_REPO: &str = "sway";
const FUEL_CORE_REPO: &str = "fuel-core";
const CHANNEL_FUEL_LATEST_TOML_URL: &str =
    "https://raw.githubusercontent.com/FuelLabs/fuelup/gh-pages/channel-fuel-latest.toml";

#[derive(Debug, Serialize, Deserialize)]
struct WorkflowRunApiResponse {
    workflow_runs: Vec<WorkflowRun>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LatestReleaseApiResponse {
    url: String,
    tag_name: String,
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TreeApiResponse {
    tree: Vec<File>,
}

#[derive(Debug, Serialize, Deserialize)]
struct File {
    path: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct WorkflowRun {
    name: String,
    head_branch: String,
    html_url: String,
}

const MAX_VERSIONS: usize = 3;

fn get_workflow_runs(repo: &str) -> Result<WorkflowRunApiResponse> {
    let github_actions_runs_api_url = format!(
        "{}{}/{}?event=release&status=success",
        GITHUB_API_REPOS_BASE_URL, repo, ACTIONS_RUNS
    );
    let handle = ureq::builder().user_agent("fuelup").build();
    let resp = handle
        .get(&github_actions_runs_api_url)
        .call()
        .unwrap_or_else(|_| panic!("Could not get workflow runs for {}", repo));

    let mut data = Vec::new();
    resp.into_reader().read_to_end(&mut data)?;

    Ok(serde_json::from_str(&String::from_utf8_lossy(&data))
        .unwrap_or_else(|_| panic!("Failed to deserialize a workflow run for repo {}", repo)))
}

fn get_latest_release_version(repo: &str) -> Result<Version> {
    let url = format!(
        "https://api.github.com/repos/FuelLabs/{}/releases/latest",
        repo
    );

    let handle = ureq::builder().user_agent("fuelup").build();
    let response: LatestReleaseApiResponse = match handle.get(&url).call() {
        Ok(r) => serde_json::from_reader(r.into_reader())?,
        Err(e) => {
            bail!("Could not get latest release for {}: {}", repo, e);
        }
    };

    let version_str = response.tag_name["v".len()..].to_string();

    let version = Version::parse(&version_str)?;
    Ok(version)
}

fn collect_new_versions(channel: &Document, repo: &str) -> Result<Vec<Version>> {
    let package_name: &str = match repo {
        SWAY_REPO => "forc",
        _ => repo,
    };
    let latest_indexed_version = parse_latest_indexed_version(channel, package_name);
    let response: WorkflowRunApiResponse = get_workflow_runs(repo)?;

    let new_versions: Vec<Version> = response
        .workflow_runs
        .iter()
        .filter(|r| r.name == "CI")
        .map_while(|r| {
            // Fine to unwrap here since branches strictly follow the format v"x.y.z"
            Version::from_str(&r.head_branch[1..])
                .unwrap()
                .gt(&latest_indexed_version)
                .then_some(Version::from_str(&r.head_branch[1..]).unwrap())
        })
        .collect();

    // In case something went wrong above, we only want a maximum of 3 versions each so that we do not start too many CI jobs.
    Ok(new_versions[..std::cmp::min(new_versions.len(), MAX_VERSIONS)].to_vec())
}

fn parse_latest_indexed_version(channel: &Document, package: &str) -> Version {
    Version::from_str(
        channel["pkg"][package]["version"]
            .as_str()
            .unwrap_or_else(|| {
                panic!(
                    "Could not parse {} version str from {} toml",
                    package, channel
                )
            }),
    )
    .unwrap_or_else(|_| panic!("Could not create version from {}", package))
}

fn fmt_versions(forc_version: &str, fuel_core_version: &str) -> String {
    format!("forc-{}@fuel-core-{}", forc_version, fuel_core_version)
}

fn print_selected_versions(forc_versions: &[Version], fuel_core_versions: &[Version]) -> String {
    let mut output = String::new();

    for forc in forc_versions {
        for fuel_core in fuel_core_versions {
            let formatted_versions = fmt_versions(&forc.to_string(), &fuel_core.to_string());
            output.push_str(&formatted_versions);
            output.push('\n');
        }
    }

    print!("{}", output);
    // Return output for testing purposes
    output.to_string()
}

fn compare_rest() -> Result<()> {
    let handle = ureq::builder().user_agent("fuelup").build();

    let toml_resp = match handle.get(CHANNEL_FUEL_LATEST_TOML_URL).call() {
        Ok(r) => r
            .into_string()
            .expect("Could not convert channel to string"),
        Err(e) => {
            bail!(
                "Unexpected error trying to fetch channel: {} - retrying at the next scheduled time",
                e
            );
        }
    };

    let channel_doc = toml_resp
        .parse::<Document>()
        .expect("invalid channel.toml parsed");

    let components = Components::collect_publishables()?;

    for component in components
        .iter()
        .filter(|c| !["forc", "fuel-core"].contains(&c.name.as_str()))
    {
        let latest_actual_version = get_latest_version(&component.repository_name)?;
        let latest_indexed_version = parse_latest_indexed_version(&channel_doc, &component.name);

        // If any of the other components are outdated, execute build-channel and exit this iteration.
        if latest_indexed_version < latest_actual_version {
            let latest_forc_indexed_version = parse_latest_indexed_version(&channel_doc, "forc");
            let latest_fuel_core_indexed_version =
                parse_latest_indexed_version(&channel_doc, "fuel-core");

            // Gives date in YYYY-MM-DD
            let date_now = time::OffsetDateTime::now_utc().date().to_string();

            println!(
                "Running build-channel with inputs: date={}, forc={}, fuel-core={}",
                &date_now, latest_forc_indexed_version, latest_fuel_core_indexed_version
            );
            // Equivalent to running:
            // build-channel channel-fuel-latest.toml <date_now> forc=<latest_forc_indexed_version> fuel-core=<latest_fuel_core_indexed_version>
            Command::new("build-channel")
                .args([
                    "channel-fuel-latest.toml",
                    &date_now,
                    &format!("forc={}", latest_forc_indexed_version),
                    &format!("fuel-core={}", latest_fuel_core_indexed_version),
                ])
                .exec();

            break;
        }
    }

    Ok(())
}

/// Get the latest version of a release using GitHub API, first trying from workflow runs API, then
/// from the releases API.
fn get_latest_version(repo: &str) -> Result<Version> {
    // We prefer using the workflow runs API because we can query for successful and completed runs, which
    // would guarantee the existence of the release binaries. Releases can be published and be available
    // without the binaries being ready yet, which causes inconsistency.
    if let Some(latest_run) = get_workflow_runs(repo)?.workflow_runs.first() {
        Ok(Version::from_str(&latest_run.head_branch[1..])?)
    } else {
        get_latest_release_version(repo)
    }
}

fn compare_compatibility() -> Result<()> {
    let handle = ureq::builder().user_agent("fuelup").build();

    let toml_resp = match handle.get(CHANNEL_FUEL_LATEST_TOML_URL).call() {
        Ok(r) => r
            .into_string()
            .expect("Could not convert channel to string"),
        Err(ureq::Error::Status(404, r)) => {
            eprintln!(
                "Error {}: Could not download channel-fuel-latest.toml from {}; re-generating channel.",
                r.status(),
                &CHANNEL_FUEL_LATEST_TOML_URL
            );

            let sway_runs = get_workflow_runs(SWAY_REPO)?;
            let fuel_core_runs = get_workflow_runs(FUEL_CORE_REPO)?;

            let latest_sway_version =
                Version::from_str(&sway_runs.workflow_runs[0].head_branch[1..]).unwrap();
            let latest_fuel_core_version =
                Version::from_str(&fuel_core_runs.workflow_runs[0].head_branch[1..]).unwrap();
            print_selected_versions(&[latest_sway_version], &[latest_fuel_core_version]);
            std::process::exit(0);
        }
        Err(e) => {
            bail!(
                "Unexpected error trying to fetch channel: {} - retrying at the next scheduled time",
                e
            );
        }
    };

    let channel_doc = toml_resp
        .parse::<Document>()
        .expect("invalid channel.toml parsed");

    let forc_versions = collect_new_versions(&channel_doc, SWAY_REPO).unwrap();
    let fuel_core_versions = collect_new_versions(&channel_doc, FUEL_CORE_REPO).unwrap();

    let versions = select_versions(&channel_doc, forc_versions, fuel_core_versions);
    print_selected_versions(&versions.0, &versions.1);
    Ok(())
}

const USAGE: &str = "Usage: compare-versions [compatibility|rest]";

fn main() -> Result<()> {
    let mut args = std::env::args();

    if args.len() != 2 {
        bail!("Incorrect number of args.\n{USAGE}");
    }

    args.next();
    match args.next().as_deref() {
        // Compare compatibility between fuel-core and forc, and republish channel if needed.
        // Note that this does not always publish the latest forc/fuel-core.
        Some("compatibility") => compare_compatibility()?,
        // Compare versions of other components, and republish channel if there are new versions.
        Some("rest") => compare_rest()?,
        Some(a) => bail!("Unrecognized arg '{}'.\n{USAGE}", a),
        None => unreachable!(),
    }
    // run()

    Ok(())
}

fn select_versions(
    channel: &Document,
    mut forc_versions: Vec<Version>,
    mut fuel_core_versions: Vec<Version>,
) -> (Vec<Version>, Vec<Version>) {
    let latest_forc_indexed_version = parse_latest_indexed_version(channel, "forc");
    let latest_fuel_core_indexed_version = parse_latest_indexed_version(channel, "fuel-core");

    match (forc_versions.is_empty(), fuel_core_versions.is_empty()) {
        (true, false) => forc_versions.push(latest_forc_indexed_version),
        (false, true) => fuel_core_versions.push(latest_fuel_core_indexed_version),
        (false, false) => {
            forc_versions.push(latest_forc_indexed_version);
            fuel_core_versions.push(latest_fuel_core_indexed_version);
        }
        (true, true) => {}
    };

    (forc_versions, fuel_core_versions)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn example_channel() -> Document {
        let channel_toml_str = r#"
[pkg.forc]
version = "0.16.2"
[pkg.forc.target.darwin_amd64]
url = "https://github.com/FuelLabs/sway/releases/download/v0.16.2/forc-binaries-darwin_amd64.tar.gz"
hash = "ce5894333926dbcbfe47c78963f153549e882590717bf57a88267daad576b307"

[pkg.fuel-core]
version = "0.9.4"
[pkg.fuel-core.target.aarch64-apple-darwin]
url = "https://github.com/FuelLabs/fuel-core/releases/download/v0.9.4/fuel-core-0.9.4-aarch64-apple-darwin.tar.gz"
hash = "17e255b3f9a293b5f6b991092d43ac19560de9091fcf2913add6958549018b0f"
"#
        .to_string();
        channel_toml_str.parse::<Document>().unwrap()
    }

    #[test]
    fn test_parse_one_each() {
        let channel_doc = example_channel();
        let expected_str = "forc-0.17.0@fuel-core-0.9.5\nforc-0.17.0@fuel-core-0.9.4\nforc-0.16.2@fuel-core-0.9.5\nforc-0.16.2@fuel-core-0.9.4\n";

        let versions = select_versions(
            &channel_doc,
            vec![Version::new(0, 17, 0)],
            vec![Version::new(0, 9, 5)],
        );
        assert_eq!(
            expected_str,
            print_selected_versions(&versions.0, &versions.1)
        )
    }

    #[test]
    fn test_parse_both_empty() {
        assert_eq!("", print_selected_versions(&[], &[]));
    }

    #[test]
    fn test_parse_empty_fuel_core_version() {
        let channel_doc = example_channel();
        let expected_str = "forc-0.16.2@fuel-core-0.9.4\nforc-0.17.0@fuel-core-0.9.4\n";

        let versions = select_versions(
            &channel_doc,
            vec![Version::new(0, 16, 2), Version::new(0, 17, 0)],
            vec![],
        );
        assert_eq!(
            expected_str,
            print_selected_versions(&versions.0, &versions.1)
        );
    }
}
