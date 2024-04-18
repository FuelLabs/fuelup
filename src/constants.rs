use time::{format_description::FormatItem, macros::format_description};

pub const FUELUP_GH_PAGES: &str = "https://raw.githubusercontent.com/FuelLabs/fuelup/gh-pages/";
pub const FUEL_TOOLCHAIN_TOML_FILE: &str = "fuel-toolchain.toml";
pub const FUELS_VERSION_FILE: &str = "fuels_version";

pub const CHANNEL_LATEST_URL: &str =
    "https://raw.githubusercontent.com/FuelLabs/fuelup/gh-pages/channel-fuel-beta-5.toml";
pub const CHANNEL_LATEST_FILE_NAME: &str = "channel-fuel-beta-5.toml";
pub const CHANNEL_NIGHTLY_FILE_NAME: &str = "channel-fuel-nightly.toml";
pub const CHANNEL_BETA_1_FILE_NAME: &str = "channel-fuel-beta-1.toml";
pub const CHANNEL_BETA_2_FILE_NAME: &str = "channel-fuel-beta-2.toml";
pub const CHANNEL_BETA_3_FILE_NAME: &str = "channel-fuel-beta-3.toml";
pub const CHANNEL_BETA_4_FILE_NAME: &str = "channel-fuel-beta-4.toml";
pub const CHANNEL_BETA_5_FILE_NAME: &str = "channel-fuel-beta-5.toml";
pub const CHANNEL_PREVIEW_FILE_NAME: &str = "channel-fuel-preview.toml";

pub const DATE_FORMAT: &[FormatItem] = format_description!("[year]-[month]-[day]");
pub const DATE_FORMAT_URL_FRIENDLY: &[FormatItem] = format_description!("[year]/[month]/[day]");
pub const VALID_CHANNEL_STR: &str =
    "<latest-YYYY-MM-DD|nightly-YYYY-MM-DD|beta-1|beta-2|beta-3|beta-4>";
