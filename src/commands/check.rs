use anyhow::Result;
use clap::Parser;

use crate::ops::fuelup_check;

#[derive(Debug, Parser)]
pub struct CheckCommand {}

pub mod plugin {
    pub const FMT: &str = "fmt";
    pub const LSP: &str = "lsp";
    pub const EXPLORE: &str = "explore";
}

pub fn exec() -> Result<()> {
    fuelup_check::check()
}
