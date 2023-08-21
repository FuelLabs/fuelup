use super::{NIX_CMD, PROFILE_LIST_ARGS};
use crate::commands::nix::nix_info;
use anyhow::{bail, Result};
use clap::Parser;
use std::{collections::HashMap, process::Command};
use tracing::info;

#[derive(Debug, Parser)]
pub struct NixListCommand;

#[derive(Debug)]
pub(crate) struct NixBinaryInfo {
    pub(crate) name: String,
    pub(crate) index: u32,
    pub(crate) unlocked_attr_path: String,
    pub(crate) locked_attr_path: String,
    pub(crate) nix_store_path: String,
}
impl NixBinaryInfo {
    fn new(
        name: &str,
        index: &str,
        unlocked_attr_path: &str,
        locked_attr_path: &str,
        nix_store_path: &str,
    ) -> Self {
        Self {
            name: name.into(),
            index: index.parse::<u32>().unwrap(),
            unlocked_attr_path: unlocked_attr_path.into(),
            locked_attr_path: unlocked_attr_path.into(),
            nix_store_path: nix_store_path.into(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct NixBinaryList(pub(crate) HashMap<String, Vec<NixBinaryInfo>>);

impl From<Vec<u8>> for NixBinaryList {
    fn from(v: Vec<u8>) -> Self {
        let mut map: HashMap<String, Vec<NixBinaryInfo>> = HashMap::new();
        let stdout = String::from_utf8_lossy(&v);
        let mut stdout_iter = stdout.split_whitespace();
        let mut count = stdout_iter.clone().count();
        let mut outer_vec = Vec::new();
        while count != 0 {
            let mut inner_vec = Vec::new();
            for _ in 0..4 {
                if let Some(val) = stdout_iter.next() {
                    inner_vec.push(val);
                }
            }
            outer_vec.push(inner_vec);
            count -= 4;
        }
        for inner_vec in outer_vec.iter() {
            let unlocked_attr_path = inner_vec[1];
            if let Some(index) = unlocked_attr_path.find(".fuel") {
                let (_, name) = unlocked_attr_path.split_at(index);
                let nix_bin = NixBinaryInfo::new(
                    name,
                    inner_vec[0],
                    unlocked_attr_path,
                    inner_vec[2],
                    inner_vec[3],
                );
                if let Some(index) = name.find('-') {
                    let (_, toolchain) = name.clone().split_at(index);
                    match map.get_mut(toolchain) {
                        Some(nix_bins) => nix_bins.push(nix_bin),
                        None => {
                            map.insert(toolchain.into(), vec![nix_bin]);
                        }
                    }
                }
            }
        }
        Self(map)
    }
}

pub fn nix_list(_command: NixListCommand) -> Result<()> {
    match Command::new(NIX_CMD).args(PROFILE_LIST_ARGS).output() {
        Ok(output) => {
            let nix_bin_list = NixBinaryList::from(output.stdout);
            println!("{nix_bin_list:?}");
            // nix_info!(output);
            Ok(())
        }
        Err(err) => bail!("failed to show installed binaries for profile: {err}"),
    }
}
