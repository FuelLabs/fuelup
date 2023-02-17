//! # `build-channel`
//!
//! This crate is a script that allows the user to generate a channel (TOML file) that
//! makes components developed by Fuel Labs available for consumption through fuelup.
//!
//! # Examples:
//!
//! To build a channel with the latest versions,
//!
//! ```sh
//! build-channel my-channel.toml 2023-02-13
//! ```
//!
//! To build a channel while specifying a specific version for a component,
//!
//! ```sh
//! build-channel my-channel.toml 2023-02-13 forc=0.35.0
//! ```
//!
//! To build a channel while specifying the GitHub run ID (used in CI) and
//! a specific version for a component,
//!
//! ```sh
//! build-channel my-channel.toml 2023-02-13 123456789 forc=0.35.0
//! ```
//!
//! To build a nightly channel,
//!
//! ```sh
//! build-channel --nightly my-channel.toml 2023-02-13
//! ```

use anyhow::{bail, Result};
use clap::Parser;
use component::{Component, Components};
use once_cell::sync::Lazy;
use semver::Version;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use sha2::Sha256;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::io::Read;
use toml_edit::value;
use toml_edit::Document;

static TODAY: Lazy<String> = Lazy::new(|| chrono::Utc::now().format("%Y%m%d").to_string());

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
    /// Specify if we are building a nightly channel
    #[clap(long, short)]
    pub nightly: bool,
    /// the TOML file name
    pub out_file: String,
    /// the publish date
    pub publish_date: String,
    /// the GitHub run ID
    #[clap(long)]
    pub github_run_id: Option<String>,
    /// key-value pairs of components and their versions to include in a channel, eg. forc=0.35.0
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
    name: String,
}

fn get_version(component: &Component) -> Result<Version> {
    let handle = ureq::builder().user_agent("fuelup").build();
    let mut data = Vec::new();

    let url = format!(
        "https://api.github.com/repos/FuelLabs/{}/releases/latest",
        component.repository_name
    );

    let resp = handle.get(&url).call()?;

    resp.into_reader().read_to_end(&mut data)?;
    let response: LatestReleaseApiResponse = serde_json::from_str(&String::from_utf8_lossy(&data))?;

    let version_str = response.tag_name["v".len()..].to_string();

    let version = Version::parse(&version_str)?;

    Ok(version)
}

fn components_exists(components: &HashMap<String, Version>) -> bool {
    components.contains_key(&"forc".to_string())
        && components.contains_key(&"fuel-core".to_string())
}

fn validate_components(nightly: bool, components: &HashMap<String, Version>) -> Result<()> {
    if nightly && !components.is_empty() {
        bail!("Cannot specify versions when building 'nightly' channel")
    } else if !nightly && !components_exists(components) {
        println!(
"warning: You are not specifying versions for 'forc' and 'fuel-core' when building a channel.
This could result in incompatibility between forc and fuel-core."
        )
    }

    Ok(())
}

/// Generates a channel TOML based on the components downloadable from the 'sway-nightly-binaries'
/// repository. This TOML should be made available by fuelup.
fn write_nightly_document(document: &mut Document, components: Vec<Component>) -> Result<()> {
    let mut data = Vec::new();
    let nightly_release_url = format!(
        "https://api.github.com/repos/FuelLabs/sway-nightly-binaries/releases/tags/nightly-{}",
        *TODAY
    );

    let resp = ureq::get(&nightly_release_url).call()?;
    resp.into_reader().read_to_end(&mut data)?;
    let release: Release = serde_json::from_str(&String::from_utf8_lossy(&data))?;

    for asset in release.assets {
        for component in &components {
            // Asset name example: fuel-core-0.15.1+nightly.20230111.a5514420e5-x86_64-unknown-linux-gnu.tar.gz
            // If an asset's name matches a component's declared tarball_prefix in components.toml,
            // we want to store the download information in a channel.
            if let Some(stripped) = asset.name.strip_prefix(&component.tarball_prefix) {
                println!("\nWriting package info for component '{}'", &component.name);

                if let Some(pkg) = document.get("pkg") {
                    if pkg.get(&component.name).is_none() {
                        document["pkg"][&component.name] = implicit_table();
                        document["pkg"][&component.name]["target"] = implicit_table();
                    }
                }

                // Example output: Some((0.15.1+nightly.20230111.a5514420e5, x86_64-unknown-linux-gnu.tar.gz))
                // We want to record the version and target in the channel toml.
                let split = stripped[1..].split_once('-');
                if let Some((version, tarball_name)) = split {
                    document["pkg"][&component.name]["version"] = value(version.to_string());

                    // Example output: Some((x86_64-unknown-linux-gnu, tar.gz))
                    if let Some((target, _)) = tarball_name.split_once('.') {
                        let mut data = Vec::new();

                        document["pkg"][&component.name]["target"][target.to_string()] =
                            implicit_table();
                        document["pkg"][&component.name]["target"][target.to_string()]["url"] =
                            value(&asset.browser_download_url);

                        if let Ok(res) = ureq::get(&asset.browser_download_url).call() {
                            res.into_reader().read_to_end(&mut data)?;
                            let mut hasher = Sha256::new();
                            hasher.update(data);
                            let actual_hash = format!("{:x}", hasher.finalize());

                            println!(
                                "url: {}\nhash: {}",
                                &asset.browser_download_url, &actual_hash
                            );
                            document["pkg"][&component.name]["target"][target.to_string()]
                                ["hash"] = value(actual_hash);
                        };
                    };
                }
            }
        }
    }
    Ok(())
}

/// Generates a channel TOML based on the components downloadable from the different Fuel Labs
/// repositories. This TOML should be made available by fuelup. You may optionally specify
/// the versions that should be included in this channel.
fn write_document(
    document: &mut Document,
    components: Vec<Component>,
    component_versions: HashMap<String, Version>,
) -> Result<()> {
    for component in components {
        println!("\nWriting package info for component '{}'", &component.name);
        let tag_prefix = if component.name == "forc" {
            "forc-binaries"
        } else {
            &component.name
        };

        let version = match component_versions.contains_key(&component.name) {
            true => component_versions[&component.name].clone(),
            false => get_version(&component)?,
        };

        let (repo, tag, tarball_prefix) = {
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
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut component_versions: HashMap<String, Version> = HashMap::new();
    for (component, version) in args.packages {
        component_versions.insert(component, version);
    }
    validate_components(args.nightly, &component_versions)?;

    let components = Components::collect_publishables()?;

    let mut document = Document::new();
    document["pkg"] = implicit_table();

    if args.nightly {
        write_nightly_document(&mut document, components)?;
    } else {
        write_document(&mut document, components, component_versions)?;
    }

    println!("writing channel: '{}'", &args.out_file);
    let mut channel_str = String::new();
    if let Some(github_run_id) = args.github_run_id {
        channel_str.push_str(&format!(
            "published_by = \"https://github.com/FuelLabs/fuelup/actions/runs/{}\"\n",
            github_run_id
        ));
    }
    channel_str.push_str(&format!("date = \"{}\"\n", args.publish_date));
    channel_str.push_str(&document.to_string());
    fs::write(&args.out_file, &channel_str)?;

    Ok(())
}
