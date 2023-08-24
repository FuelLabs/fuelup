//! Utility for translating user commands into flake links,
//! and flake links into display messages or internal debug info
//! for handling toolchain or component management for the user automatically.

use super::{install::NixInstallCommand, list::UnlockedFlakeURL, FUEL_NIX_LINK};
use crate::commands::fuelup::FuelupCommand;
use anyhow::{bail, Result};

/// Handles getting toolchain or component information to translate
/// into fuel.nix flake links and info presented to the user.
pub(crate) trait FlakeLinkInfo {
    /// the name of a toolchain or component
    fn name(&self) -> String;
    fn get_toolchain(&self) -> Result<FuelToolchain> {
        FuelToolchain::from_str(self.name())
    }
    fn get_component(&self) -> Result<(FuelComponent, FuelToolchain)> {
        FuelComponent::from_str_with_toolchain(self.name())
    }
    fn is_toolchain(&self) -> bool {
        FuelToolchain::from_str(self.name()).is_ok()
    }
    fn is_component(&self) -> bool {
        FuelComponent::from_str_with_toolchain(self.name()).is_ok()
    }
}
/// Create toolchain and component links for the fuel.nix flake.
pub(crate) trait CachixLinkGenerator: FlakeLinkInfo {
    fn flake_link_toolchain_suffix(&self) -> Result<&str> {
        Ok(match self.get_toolchain()? {
            FuelToolchain::Latest => "fuel",
            FuelToolchain::Nightly => "fuel-nightly",
            FuelToolchain::Beta1 => "fuel-beta-1",
            FuelToolchain::Beta2 => "fuel-beta-2",
            FuelToolchain::Beta3 => "fuel-beta-3",
            FuelToolchain::Beta4rc => "fuel-beta-4-rc",
            FuelToolchain::Unknown => bail!("available distributed toolchains:\n  -latest\n  -nightly\n  -beta-1\n  -beta-2\n  -beta-3\n  -beta-4-rc")
        })
    }
    fn flake_link_component_suffix(&self) -> Result<(&str, &str)> {
        let (comp, tool) = self.get_component()?;
        let comp = match comp {
            FuelComponent::FuelCore => "fuel-core",
            FuelComponent::FuelCoreClient => "fuel-core-client",
            FuelComponent::FuelIndexer => "fuel-indexer",
            FuelComponent::Forc => "forc",
            FuelComponent::ForcClient => "forc-client",
            FuelComponent::ForcDoc => "forc-doc",
            FuelComponent::ForcExplore => "forc-explore",
            FuelComponent::ForcFmt => "forc-fmt",
            FuelComponent::ForcIndex => "forc-index",
            FuelComponent::ForcLsp => "forc-lsp",
            FuelComponent::ForcTx => "forc-tx",
            FuelComponent::ForcWallet => "forc-wallet",
            FuelComponent::SwayVim => "sway-vim",
        };
        let tool = match tool {
            FuelToolchain::Latest => "",
            FuelToolchain::Nightly => "-nightly",
            FuelToolchain::Beta1 => "-beta-1",
            FuelToolchain::Beta2 => "-beta-2",
            FuelToolchain::Beta3 => "-beta-3",
            FuelToolchain::Beta4rc => "-beta-4-rc",
            FuelToolchain::Unknown => bail!("available distributed toolchains:\n  -latest\n  -nightly\n  -beta-1\n  -beta-2\n  -beta-3\n  -beta-4-rc")
        };
        Ok((comp, tool))
    }
    fn flake_toolchain_link(&self) -> Result<String> {
        Ok(format!(
            "{FUEL_NIX_LINK}#{}",
            self.flake_link_toolchain_suffix()?
        ))
    }
    fn flake_component_link(&self) -> Result<String> {
        let (comp, tool) = self.flake_link_component_suffix()?;
        Ok(format!("{FUEL_NIX_LINK}#{}{}", comp, tool))
    }
}

impl FlakeLinkInfo for NixInstallCommand {
    fn name(&self) -> String {
        self.name.clone()
    }
}
impl FlakeLinkInfo for UnlockedFlakeURL {
    fn name(&self) -> String {
        let (comp, _) = split_at_toolchain(self.0.clone())
            .expect("failed to split whitespace of unlocked attribute path");
        if let Some(index) = comp.find(".fuel") {
            let (_, comp) = comp.split_at(index);
            if comp == ".fuel-" {
                // return the full toolchain name
                let comp = comp.replace('.', "");
                let comp = comp.replace('-', "");
                comp.to_string()
            } else if comp == ".fuel" {
                let comp = comp.replace('.', "");
                comp.to_string()
            } else {
                self.0.clone()
            }
        } else {
            self.0.clone()
        }
    }
}
impl CachixLinkGenerator for NixInstallCommand {}

#[derive(Eq, PartialEq, Debug, Hash)]
pub(crate) enum FuelToolchain {
    Latest,
    Nightly,
    Beta1,
    Beta2,
    Beta3,
    Beta4rc,
    Unknown,
}

impl FuelToolchain {
    fn from_str(s: String) -> Result<Self> {
        Ok(match s.to_lowercase().as_str() {
            "latest" | "fuel" => Self::Latest,
            "nightly" | "fuel-nightly" => Self::Nightly,
            "beta-1" | "beta1" | "fuel-beta-1" => Self::Beta1,
            "beta-2" | "beta2" | "fuel-beta-2" => Self::Beta2,
            "beta-3" | "beta3" | "fuel-beta-3" => Self::Beta3,
            "beta-4-rc" | "beta-4rc" | "beta4rc" | "fuel-beta-4-rc" => Self::Beta4rc,
            _ => bail!("available distributed toolchains:\n  -latest\n  -nightly\n  -beta-1\n  -beta-2\n  -beta-3\n  -beta-4-rc") 
        })
    }
    fn is_latest(&self) -> bool {
        *self == FuelToolchain::Latest
    }
}
impl From<String> for FuelToolchain {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "latest" => Self::Latest,
            "nightly" => Self::Nightly,
            "beta-1" | "beta1" => Self::Beta1,
            "beta-2" | "beta2" => Self::Beta2,
            "beta-3" | "beta3" => Self::Beta3,
            "beta-4-rc" | "beta-4rc" | "beta4rc" => Self::Beta4rc,
            _ => Self::Unknown,
        }
    }
}
impl From<FuelToolchain> for &str {
    fn from(ft: FuelToolchain) -> &'static str {
        match ft {
            FuelToolchain::Latest => "latest",
            FuelToolchain::Nightly => "nightly",
            FuelToolchain::Beta1 => "beta-1",
            FuelToolchain::Beta2 => "beta-2",
            FuelToolchain::Beta3 => "beta-3",
            FuelToolchain::Beta4rc => "beta-4-rc",
            FuelToolchain::Unknown => "unknown",
        }
    }
}

const DIST_COMPONENTS: &[FuelComponent; 13] = &[
    FuelComponent::FuelCore,
    FuelComponent::FuelCoreClient,
    FuelComponent::FuelIndexer,
    FuelComponent::Forc,
    FuelComponent::ForcClient,
    FuelComponent::ForcDoc,
    FuelComponent::ForcExplore,
    FuelComponent::ForcFmt,
    FuelComponent::ForcIndex,
    FuelComponent::ForcLsp,
    FuelComponent::ForcTx,
    FuelComponent::ForcWallet,
    FuelComponent::SwayVim,
];

// ...
//
// bail!("available distrubuted components: {err_str}\n")

#[derive(Debug)]
pub(crate) enum FuelComponent {
    FuelCore,
    FuelCoreClient,
    FuelIndexer,
    Forc,
    ForcClient,
    ForcDoc,
    ForcExplore,
    ForcFmt,
    ForcIndex,
    ForcLsp,
    ForcTx,
    ForcWallet,
    SwayVim,
}
impl FuelComponent {
    fn from_str_with_toolchain(s: String) -> Result<(Self, FuelToolchain)> {
        let (raw_comp_str, tool) = split_at_toolchain(s.to_lowercase())?;
        // remove the excess '-' between the comp and toolchain vers
        let comp_str = if !tool.is_latest() {
            let mut comp_str = raw_comp_str.chars();
            comp_str.next_back();
            comp_str.collect::<String>()
        } else {
            raw_comp_str
        };
        let comp = Self::from_str(comp_str)?;
        Ok((comp, tool))
    }

    fn from_str(comp_str: String) -> Result<Self> {
        match comp_str.as_str() {
            "fuel-core" => Ok(Self::FuelCore),
            "fuel-core-client" => Ok(Self::FuelCoreClient),
            "fuel-indexer" => Ok(Self::FuelIndexer),
            "forc" => Ok(Self::Forc),
            "forc-client" => Ok(Self::ForcClient),
            "forc-doc" => Ok(Self::ForcDoc),
            "forc-explore" => Ok(Self::ForcExplore),
            "forc-fmt" => Ok(Self::ForcFmt),
            "forc-index" => Ok(Self::ForcIndex),
            "forc-lsp" => Ok(Self::ForcLsp),
            "forc-tx" => Ok(Self::ForcTx),
            "forc-wallet" => Ok(Self::ForcWallet),
            "sway-vim" => Ok(Self::SwayVim),
            _ => {
                let available_components = DIST_COMPONENTS
                    .iter()
                    .map(|comp| comp.as_display_str())
                    .collect::<Vec<&str>>()
                    .join("\n");
                bail!("available distrubuted components:\n  {available_components}\n

available distributed toolchains:\n  -latest\n  -nightly\n  -beta-1\n  -beta-2\n  -beta-3\n  -beta-4-rc

please form a valid component, like so: fuel-core-beta-3"
                )
            }
        }
    }

    fn as_display_str(&self) -> &'static str {
        match self {
            FuelComponent::FuelCore => "- fuel-core",
            FuelComponent::FuelCoreClient => "- fuel-core-client",
            FuelComponent::FuelIndexer => "- fuel-indexer",
            FuelComponent::Forc => "- forc",
            FuelComponent::ForcClient => "- forc-client",
            FuelComponent::ForcDoc => "- forc-doc",
            FuelComponent::ForcExplore => "- forc-explore",
            FuelComponent::ForcFmt => "- forc-fmt",
            FuelComponent::ForcIndex => "- forc-index",
            FuelComponent::ForcLsp => "- forc-lsp",
            FuelComponent::ForcTx => "- forc-tx",
            FuelComponent::ForcWallet => "- forc-wallet",
            FuelComponent::SwayVim => "- sway-vim",
        }
    }
}
pub(crate) fn split_at_toolchain(s: String) -> Result<(String, FuelToolchain)> {
    let (comp, tool) = if let Some(index) = s.find("beta") {
        let (comp, tool) = s.split_at(index);
        (comp.into(), FuelToolchain::from_str(tool.to_string())?)
    } else if let Some(index) = s.find("nightly") {
        let (comp, tool) = s.split_at(index);
        (comp.into(), FuelToolchain::from_str(tool.to_string())?)
    } else {
        (s, FuelToolchain::Latest)
    };
    Ok((comp, tool))
}
