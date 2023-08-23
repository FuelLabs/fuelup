use crate::commands::fuelup::FuelupCommand;

use super::{install::NixInstallCommand, list::UnlockedFlakeURL, FUEL_NIX_LINK};
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
        FuelComponent::from_str(self.name())
    }
    fn is_toolchain(&self) -> bool {
        FuelToolchain::from_str(self.name()).is_ok()
    }
    fn is_component(&self) -> bool {
        FuelComponent::from_str(self.name()).is_ok()
    }
}
/// Create toolchain and component links for the fuel.nix flake.
pub(crate) trait CachixLinkGenerator: FlakeLinkInfo {
    fn nix_toolchain_suffix(&self) -> Result<&str> {
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
    fn nix_component_suffix(&self) -> Result<(&str, &str)> {
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
    fn toolchain_link(&self) -> Result<String> {
        Ok(format!("{FUEL_NIX_LINK}#{}", self.nix_toolchain_suffix()?))
    }
    fn component_link(&self) -> Result<String> {
        let (comp, tool) = self.nix_component_suffix()?;
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
impl CachixLinkGenerator for UnlockedFlakeURL {}

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
    fn from_str(s: String) -> Result<(Self, FuelToolchain)> {
        let (comp, tool) = split_at_toolchain(s.to_lowercase())?;
        // remove the excess '-' between the comp and toolchain vers
        let comp = if !tool.is_latest() {
            let mut comp = comp.chars();
            comp.next_back();
            comp.collect::<String>()
        } else {
            comp
        };
        Ok((match comp.as_str() {
            "fuel-core" => Self::FuelCore,
            "fuel-core-client" => Self::FuelCoreClient,
            "fuel-indexer" => Self::FuelIndexer,
            "forc" => Self::Forc,
            "forc-client" => Self::ForcClient,
            "forc-doc" => Self::ForcDoc,
            "forc-explore" => Self::ForcExplore,
            "forc-fmt" => Self::ForcFmt,
            "forc-index" => Self::ForcIndex,
            "forc-lsp" => Self::ForcLsp,
            "forc-tx" => Self::ForcTx,
            "forc-wallet" => Self::ForcWallet,
            "sway-vim" => Self::SwayVim,
            _ => bail!(
                "available distrubuted components:\n  -fuel-core\n  -fuel-core-client\n  -fuel-indexer\n  -forc\n  -forc-client\n  -forc-doc\n  -forc-explore\n  -forc-fmt\n  -forc-index\n  -forc-lsp\n  -forc-tx\n  -forc-wallet\n  -sway-vim\n
available distributed toolchains:\n  -latest\n  -nightly\n  -beta-1\n  -beta-2\n  -beta-3\n  -beta-4-rc

please form a valid component, like so: fuel-core-beta-3"
            )
        }, tool))
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
