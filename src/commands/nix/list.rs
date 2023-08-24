use super::{
    flake_utils::{split_at_toolchain, FlakeLinkInfo, FuelToolchain},
    NIX_CMD, PROFILE_LIST_ARGS,
};
use crate::commands::nix::nix_info;
use anyhow::{bail, Result};
use clap::Parser;
use std::{collections::HashMap, process::Command};
use tracing::info;

#[derive(Debug, Parser)]
pub struct NixListCommand;

/// A binary package installed by the fuel.nix flake.
#[derive(Debug)]
pub(crate) struct NixBinaryInfo {
    pub(crate) name: String,
    pub(crate) index: u32,
    pub(crate) flake_attribute: Option<String>,
    pub(crate) unlocked_flake_url: String,
    pub(crate) locked_flake_url: String,
    pub(crate) nix_store_path: String,
}
impl NixBinaryInfo {
    fn new(
        name: String,
        index: &str,
        flake_attribute: Option<String>,
        unlocked_flake_url: &str,
        locked_flake_url: &str,
        nix_store_path: &str,
    ) -> Self {
        Self {
            name,
            index: index.parse::<u32>().unwrap(),
            flake_attribute,
            unlocked_flake_url: unlocked_flake_url.into(),
            locked_flake_url: unlocked_flake_url.into(),
            nix_store_path: nix_store_path.into(),
        }
    }
}

/// Used to get the information about a package since it has a
/// more reliable structure than either the flake attribute, which
/// isn't present on my machine (M1 macbook pro, nix (Nix) 2.15.0),
/// or the nix store path which contains a randomly produced hash.
pub(crate) struct UnlockedFlakeURL(pub(crate) String);
impl<'a> From<&'a str> for UnlockedFlakeURL {
    fn from(s: &'a str) -> Self {
        Self(s.to_string())
    }
}
impl UnlockedFlakeURL {
    fn split_at_toolchain(&self) -> Result<(String, FuelToolchain)> {
        if let Some(index) = self.0.find(".fuel") {
            let (_, tool) = split_at_toolchain(self.0.split_at(index).1.split_at(1).1.to_string())
                .expect("failed to get toolchain from unlocked attribute path");
            Ok((self.name(), tool))
        } else {
            bail!("could not get toolchain info from attribute path")
        }
    }
    fn split_at_component(&self) -> Result<(String, FuelToolchain)> {
        if let Some(index) = self.0.find(".forc") {
            split_at_toolchain(self.0.split_at(index).1.split_at(1).1.to_string())
        } else {
            bail!("could not get toolchain info from attribute path")
        }
    }
}

#[derive(Debug)]
pub(crate) struct NixBinaryList(pub(crate) HashMap<FuelToolchain, Vec<NixBinaryInfo>>);

/// Currently this collects a static 4 values from the stdout string
/// produced by `nix profile list`, however, in some cases we may actually
/// get 5. The four present are the index, unlocked flake link, locked flake link
/// and nix store path. The fifth would come after the index and is the flake
/// attribute which for some reason doesn't show up presently but _could_ in the
/// future.
///
/// To avoid breakage we could look to collect at every index, then to be sure
/// of each value we can perform checks to see what that data holds. eg, an index
/// can be parsed as an integer, the flake links will start with "github:fuellabs" and
/// the nix store path will start with "nix/store/".
impl From<Vec<u8>> for NixBinaryList {
    fn from(v: Vec<u8>) -> Self {
        let mut map: HashMap<FuelToolchain, Vec<NixBinaryInfo>> = HashMap::new();
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
            let unlocked_attr_path = UnlockedFlakeURL::from(inner_vec[1]);
            if unlocked_attr_path.is_toolchain() {
                let (name, toolchain) = unlocked_attr_path
                    .split_at_toolchain()
                    .expect("failed to get pkg info from unlocked attribute path");
                let nix_bin = NixBinaryInfo::new(
                    name,
                    inner_vec[0],
                    None, /* see comment on this impl */
                    inner_vec[1],
                    inner_vec[2],
                    inner_vec[3],
                );

                match map.get_mut(&toolchain) {
                    Some(nix_bins) => nix_bins.push(nix_bin),
                    None => {
                        map.insert(toolchain, vec![nix_bin]);
                    }
                }
            } else if unlocked_attr_path.is_component() {
                let (name, toolchain) = unlocked_attr_path
                    .split_at_component()
                    .expect("failed to get pkg info from unlocked attribute path");
                let nix_bin = NixBinaryInfo::new(
                    name,
                    inner_vec[0],
                    None, /* see comment on this impl */
                    inner_vec[1],
                    inner_vec[2],
                    inner_vec[3],
                );

                match map.get_mut(&toolchain) {
                    Some(nix_bins) => nix_bins.push(nix_bin),
                    None => {
                        map.insert(toolchain, vec![nix_bin]);
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
            dbg!(nix_bin_list);
            // nix_info!(output);
            Ok(())
        }
        Err(err) => bail!("failed to show installed binaries for profile: {err}"),
    }
}
