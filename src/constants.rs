use time::{format_description::FormatItem, macros::format_description};

pub const FUELUP_GH_PAGES: &str = "https://raw.githubusercontent.com/FuelLabs/fuelup/gh-pages/";

pub const CHANNEL_LATEST_URL: &str =
    "https://raw.githubusercontent.com/FuelLabs/fuelup/gh-pages/channel-fuel-latest.toml";
pub const CHANNEL_LATEST_FILE_NAME: &str = "channel-fuel-latest.toml";
pub const CHANNEL_NIGHTLY_FILE_NAME: &str = "channel-fuel-nightly.toml";
pub const CHANNEL_BETA_1_FILE_NAME: &str = "channel-fuel-beta-1.toml";

pub const DATE_FORMAT: &[FormatItem] = format_description!("[year]-[month]-[day]");
pub const DATE_FORMAT_URL_FRIENDLY: &[FormatItem] = format_description!("[year]/[month]/[day]");
