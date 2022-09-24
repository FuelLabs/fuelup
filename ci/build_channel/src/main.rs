use std::collections::hash_map::Keys;
use std::collections::{BTreeMap, HashMap};
use std::error::Error;

use anyhow::{bail, Result};
use clap::Parser;
use component::Components;
use semver::Version;
use serde::{Deserialize, Serialize};
use toml_edit::value;

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
    pub out: String,
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

fn get_version(channel: &str) -> Version {
    match channel {
        "nightly" => {}
        "latest" => {}
        _ => bail!("Invalid channel '{channel}'"),
    }
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

    println!("{}", args.channel);
    let mut component_versions: HashMap<String, Version> = HashMap::new();

    for package in args.packages {
        component_versions.insert(package.0, package.1);
    }

    validate_components(&args.channel, &component_versions)?;

    let components = Components::collect_publishables()?;
    let mut document = toml_edit::Document::new();
    document["pkg"] = implicit_table();

    for component in components {
        let version = match component_versions.contains_key(&component.name) {
            true => component_versions[&component.name].clone(),
            false => get_version(&args.channel),
        };

        document["pkg"][&component.name] = implicit_table();
        document["pkg"][&component.name]["version"] = value(version.to_string());

        document["pkg"][&component.name]["target"] = implicit_table();

        for target in &component.targets {
            document["pkg"][&component.name]["target"][target.to_string()] = implicit_table();
            document["pkg"][&component.name]["target"][target.to_string()]["url"] =
                value(component.download_url.clone());
            //document["pkg"][&component.name]["target"][target.to_string()]["hash"] =
            //value(bin.hash.clone());
        }
    }

    println!("{}", document);

    Ok(())
}
