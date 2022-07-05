use anyhow::Result;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::{io::Read, str::FromStr};
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
struct TreeApiResponse {
    tree: Vec<File>,
}

#[derive(Debug, Serialize, Deserialize)]
struct File {
    path: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct WorkflowRun {
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
        .expect(&format!("Could not get workflow runs for {}", repo));

    let mut data = Vec::new();
    resp.into_reader().read_to_end(&mut data)?;

    Ok(
        serde_json::from_str(&String::from_utf8_lossy(&data)).expect(&format!(
            "Failed to deserialize a workflow run for repo {}",
            repo
        )),
    )
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
    Version::from_str(channel["pkg"][package]["version"].as_str().expect(&format!(
        "Could not parse {} version str from {} toml",
        package, channel
    )))
    .expect(&format!("Could not create version from {}", package))
}

fn fmt_versions(forc_versions: &str, fuel_core_versions: &str) -> String {
    format!("[{}]\n[{}]", forc_versions, fuel_core_versions)
}

fn print_selected_versions<'a>(
    forc_versions: &mut Vec<Version>,
    fuel_core_versions: &mut Vec<Version>,
) -> String {
    let forc_versions_str = forc_versions
        .iter()
        .map(|v| "\"".to_owned() + &v.to_string() + "\",")
        .collect::<String>();
    let fuel_core_versions_str = fuel_core_versions
        .iter()
        .map(|v| "\"".to_owned() + &v.to_string() + "\",")
        .collect::<String>();
    let output = fmt_versions(
        forc_versions_str.trim_end_matches(','),
        fuel_core_versions_str.trim_end_matches(','),
    );

    print!("{}", output);
    // Return output solely for testing purposes
    output.to_string()
}

fn main() -> Result<()> {
    let handle = ureq::builder().user_agent("fuelup").build();

    let toml_resp = match handle.get(&CHANNEL_FUEL_LATEST_TOML_URL).call() {
        Ok(r) => r
            .into_string()
            .expect("Could not convert channel to string"),
        Err(_) => {
            eprintln!(
                "Could not download channel-fuel-latest.toml from {}; re-generating channel.",
                &CHANNEL_FUEL_LATEST_TOML_URL
            );

            let sway_runs = get_workflow_runs(SWAY_REPO)?;
            let fuel_core_runs = get_workflow_runs(FUEL_CORE_REPO)?;

            let latest_sway_version = &sway_runs.workflow_runs[0].head_branch[1..];
            let latest_fuel_core_version = &fuel_core_runs.workflow_runs[0].head_branch[1..];
            print_selected_versions(
                &mut vec![Version::from_str(latest_sway_version).unwrap()],
                &mut vec![Version::from_str(latest_fuel_core_version).unwrap()],
            );
            std::process::exit(0);
        }
    };

    let channel_doc = toml_resp
        .parse::<Document>()
        .expect("invalid channel.toml parsed");

    let forc_versions = collect_new_versions(&channel_doc, SWAY_REPO).unwrap();
    let fuel_core_versions = collect_new_versions(&channel_doc, FUEL_CORE_REPO).unwrap();

    select_versions(&channel_doc, forc_versions, fuel_core_versions);
    Ok(())
}

fn select_versions(
    channel: &Document,
    mut forc_versions: Vec<Version>,
    mut fuel_core_versions: Vec<Version>,
) -> String {
    let latest_forc_indexed_version = parse_latest_indexed_version(channel, "forc");
    let latest_fuel_core_indexed_version = parse_latest_indexed_version(channel, "fuel-core");

    match (forc_versions.is_empty(), fuel_core_versions.is_empty()) {
        (true, false) => forc_versions.push(latest_forc_indexed_version),
        (false, true) => fuel_core_versions.push(latest_fuel_core_indexed_version),
        (false, false) => {
            forc_versions.push(latest_forc_indexed_version);
            fuel_core_versions.push(latest_fuel_core_indexed_version);
        }
        _ => return "".to_string(),
    };

    print_selected_versions(&mut forc_versions, &mut fuel_core_versions)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn example_channel() -> String {
        r#"
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
        .to_string()
    }

    #[test]
    fn test_parse_one_each() {
        let channel_doc = example_channel().parse::<Document>().expect("Invalid doc");
        let expected_str = "[\"0.17.0\",\"0.16.2\"]\n[\"0.9.5\",\"0.9.4\"]";

        assert_eq!(
            expected_str,
            select_versions(
                &channel_doc,
                vec![Version::new(0, 17, 0)],
                vec![Version::new(0, 9, 5)]
            )
        )
    }

    #[test]
    fn test_parse_both_empty() {
        let channel_doc = example_channel().parse::<Document>().expect("Invalid doc");

        assert_eq!("", select_versions(&channel_doc, vec![], vec![]));
    }

    #[test]
    fn test_parse_empty_version() {
        let channel_doc = example_channel().parse::<Document>().expect("Invalid doc");

        let expected_str = "[\"0.16.2\",\"0.17.0\"]\n[\"0.9.4\"]";

        assert_eq!(
            expected_str,
            select_versions(
                &channel_doc,
                vec![Version::new(0, 16, 2), Version::new(0, 17, 0)],
                vec![]
            )
        );
    }
}
