use anyhow::{bail, Result};
use clap::Parser;
use component::{Component, Components};
use once_cell::sync::Lazy;
use semver::Version;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use sha2::Sha256;
use std::collections::{BTreeMap, HashMap};
use std::error::Error;
use std::fs;
use std::io::Read;
use toml_edit::value;

static TODAY: Lazy<String> = Lazy::new(|| chrono::Utc::now().format("%Y%m%d").to_string());
static TODAY_ISO: Lazy<String> =
    Lazy::new(|| format!("{}T00:00:00Z", chrono::Utc::now().format("%Y-%m-%d")));

/// Parse a single key-value pair
fn parse_key_val<T, U>(s: &str) -> Result<(T, U), Box<dyn Error + Send + Sync + 'static>>
where
    T: std::str::FromStr,
    T::Err: Error + Send + Sync + 'static,
    U: std::str::FromStr,
    U::Err: Error + Send + Sync + 'static,
{
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{}`", s))?;
    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}

#[derive(Debug, Parser)]
struct Args {
    /// Component name [possible values: latest, nightly]
    pub channel: String,
    /// the TOML file name
    pub out_file: String,
    /// the GitHub run ID
    pub github_run_id: String,
    /// the publish date
    pub publish_date: String,
    /// Component name [possible values: latest]
    #[clap(value_parser = parse_key_val::<String, Version>)]
    pub packages: Vec<(String, Version)>,
}

fn implicit_table() -> toml_edit::Item {
    let mut tbl = toml_edit::Table::new();
    tbl.set_implicit(true);
    toml_edit::Item::Table(tbl)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HashedBinary {
    pub url: String,
    pub hash: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Channel {
    pub pkg: BTreeMap<String, Package>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Package {
    pub target: BTreeMap<String, HashedBinary>,
    pub version: Version,
}

#[derive(Debug, Serialize, Deserialize)]
struct LatestReleaseApiResponse {
    url: String,
    tag_name: String,
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Release {
    assets: Vec<Asset>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Asset {
    browser_download_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Commit {
    sha: String,
}

fn get_version(channel: &str, component: &Component) -> Result<Version> {
    let handle = ureq::builder().user_agent("fuelup").build();
    let mut data = Vec::new();

    let url = format!(
        "https://api.github.com/repos/FuelLabs/{}/releases/latest",
        component.repository_name
    );

    let resp = handle.get(&url).call()?;

    resp.into_reader().read_to_end(&mut data)?;

    let response: LatestReleaseApiResponse = serde_json::from_str(&String::from_utf8_lossy(&data))?;

    let mut version_str = response.tag_name["v".len()..].to_string();

    if channel == "nightly" {
        let commit_api = format!(
            "https://api.github.com/repos/FuelLabs/{}/commits",
            component.repository_name
        );
        let resp = handle.get(&commit_api).query("until", &TODAY_ISO).call()?;

        let mut data2 = Vec::new();
        resp.into_reader().read_to_end(&mut data2)?;
        let commits: Vec<Commit> = serde_json::from_str(&String::from_utf8_lossy(&data2))?;

        let short_sha = commits[0].sha.chars().take(10).collect::<String>();

        let build_metadata = format!("{}.{}.{}", "nightly", TODAY.to_string(), short_sha);
        version_str = format!("{}+{}", version_str, build_metadata);
    };

    let version = Version::parse(&version_str)?;

    Ok(version)
}

fn components_exists(components: &HashMap<String, Version>) -> bool {
    components.contains_key(&"forc".to_string())
        && components.contains_key(&"fuel-core".to_string())
}

fn validate_components(channel: &str, components: &HashMap<String, Version>) -> Result<()> {
    match channel {
        "nightly" => {
            if !components.is_empty() {
                bail!("Cannot specify versions when building 'nightly' channel")
            }
        }
        "latest" => {
            if !components_exists(components) {
                bail!("You must specify versions for 'forc' and 'fuel-core' when building 'latest' channel")
            }
        }
        _ => bail!("Invalid channel '{channel}'"),
    }

    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut component_versions: HashMap<String, Version> = HashMap::new();

    for package in args.packages {
        component_versions.insert(package.0, package.1);
    }

    validate_components(&args.channel, &component_versions)?;

    let components = Components::collect_publishables()?;

    let mut document = toml_edit::Document::new();
    document["pkg"] = implicit_table();

    for component in components {
        println!("\nWriting package info for component '{}'", &component.name);
        let tag_prefix = if component.name == "forc" {
            "forc-binaries"
        } else {
            &component.name
        };

        let version = match component_versions.contains_key(&component.name) {
            true => component_versions[&component.name].clone(),
            false => get_version(&args.channel, &component)?,
        };

        let (repo, tag, tarball_prefix) = if args.channel.as_str() == "latest" {
            let tarball_prefix = if tag_prefix == "forc-binaries" {
                tag_prefix.to_string()
            } else {
                format!("{}-{}", tag_prefix, version)
            };
            (
                component.repository_name,
                "v".to_owned() + &version.to_string(),
                tarball_prefix,
            )
        } else {
            (
                "sway-nightly-binaries".to_string(),
                format!("nightly-{}", TODAY.to_string()),
                format!("{}-{}", tag_prefix, version),
            )
        };

        document["pkg"][&component.name] = implicit_table();
        document["pkg"][&component.name]["version"] = value(version.to_string());
        document["pkg"][&component.name]["target"] = implicit_table();

        for target in &component.targets {
            println!("Adding url and hash for target '{}'", &target);

            let mut data = Vec::new();
            let url = format!(
                "https://github.com/FuelLabs/{}/releases/download/{}/{}-{}.tar.gz",
                repo, tag, tarball_prefix, target
            );

            match ureq::get(&url).call() {
                Ok(res) => {
                    res.into_reader().read_to_end(&mut data)?;
                    let mut hasher = Sha256::new();
                    hasher.update(data);
                    let actual_hash = format!("{:x}", hasher.finalize());
                    println!("url: {}\nhash: {}", &url, &actual_hash);

                    document["pkg"][&component.name]["target"][target.to_string()] =
                        implicit_table();
                    document["pkg"][&component.name]["target"][target.to_string()]["url"] =
                        value(url);
                    document["pkg"][&component.name]["target"][target.to_string()]["hash"] =
                        value(actual_hash);
                }
                Err(e) => {
                    eprintln!("Error adding url and hash for target '{}':\n{}", target, e);
                }
            };
        }
    }

    println!("writing channel: '{}'", &args.out_file);

    let mut channel_str = String::new();
    channel_str.push_str(&format!(
        "published_by = \"https://github.com/FuelLabs/fuelup/actions/runs/{}\"\n",
        args.github_run_id
    ));
    channel_str.push_str(&format!("date = \"{}\"\n", args.publish_date));
    channel_str.push_str(&document.to_string());
    fs::write(&args.out_file, channel_str.to_string())?;

    Ok(())
}
