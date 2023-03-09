use anyhow::Result;
use serde::Deserialize;
use std::io::Read;
use tracing::info;

use crate::{commands::toolchain::ListRevisionsCommand, download::build_agent};

#[derive(Debug, Deserialize)]
struct Content {
    name: String,
}

/// Helper function to strip the following channel name from
/// channel-fuel-latest-2023-01-27.toml -> latest-2023-01-27.
/// If this fails, default to the original name.
fn strip_channel_name(name: &str) -> String {
    name.strip_prefix("channel-fuel-")
        .and_then(|s| s.strip_suffix(".toml"))
        .unwrap_or(name)
        .to_string()
}

pub fn list_revisions(_command: ListRevisionsCommand) -> Result<()> {
    let handle = build_agent()?;

    let mut data = Vec::new();

    let resp = handle
        .get("https://api.github.com/repos/fuellabs/fuelup/contents/channels/latest?ref=gh-pages")
        .call()?;

    resp.into_reader().read_to_end(&mut data)?;
    let contents: Vec<Content> = serde_json::from_slice(&data)?;

    let revisions = contents
        .iter()
        .rev()
        .map(|c| strip_channel_name(&c.name) + "\n")
        .collect::<String>();

    info!("\n'latest' revisions available:\n{}\nfor the full list, visit: https://github.com/FuelLabs/fuelup/tree/gh-pages/channels/latest", revisions);
    Ok(())
}
