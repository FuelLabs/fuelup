use time::{format_description::FormatItem, macros::format_description};

pub const SWAY_RELEASE_DOWNLOAD_URL: &str = "https://github.com/FuelLabs/sway/releases/download";
pub const FORC_CLIENT_RELEASE_DOWNLOAD_URL: &str =
    "https://github.com/FuelLabs/forc-client/releases/download";
pub const FORC_WALLET_RELEASE_DOWNLOAD_URL: &str =
    "https://github.com/FuelLabs/forc-wallet/releases/download";
pub const FUELUP_RELEASE_DOWNLOAD_URL: &str =
    "https://github.com/FuelLabs/fuelup/releases/download";
pub const FUEL_CORE_RELEASE_DOWNLOAD_URL: &str =
    "https://github.com/FuelLabs/fuel-core/releases/download";
pub const FUELUP_GH_PAGES: &str = "https://raw.githubusercontent.com/FuelLabs/fuelup/gh-pages/";

pub const CHANNEL_LATEST_URL: &str =
    "https://raw.githubusercontent.com/FuelLabs/fuelup/gh-pages/channel-fuel-latest.toml";
pub const CHANNEL_LATEST_FILE_NAME: &str = "channel-fuel-latest.toml";
pub const CHANNEL_NIGHTLY_FILE_NAME: &str = "channel-fuel-nightly.toml";

pub const DATE_FORMAT: &[FormatItem] = format_description!("[year]-[month]-[day]");
