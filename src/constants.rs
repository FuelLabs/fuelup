use time::{format_description::FormatItem, macros::format_description};

pub const SWAY_REPO: &str = "sway";
pub const FUEL_CORE_REPO: &str = "fuel-core";
pub const FUELUP_REPO: &str = "fuelup";

pub const GITHUB_API_REPOS_BASE_URL: &str = "https://api.github.com/repos/FuelLabs/";
pub const RELEASES_LATEST: &str = "releases/latest";
pub const RELEASES_TAGS: &str = "releases/tags";

pub const SWAY_RELEASE_DOWNLOAD_URL: &str = "https://github.com/FuelLabs/sway/releases/download";
pub const FUELUP_RELEASE_DOWNLOAD_URL: &str =
    "https://github.com/FuelLabs/fuelup/releases/download";
pub const FUEL_CORE_RELEASE_DOWNLOAD_URL: &str =
    "https://github.com/FuelLabs/fuel-core/releases/download";
pub const FUELUP_GH_PAGES: &str = "https://raw.githubusercontent.com/FuelLabs/fuelup/gh-pages/";

pub const CHANNEL_LATEST_FILE_NAME: &str = "channel-fuel-latest.toml";
pub const CHANNEL_NIGHTLY_FILE_NAME: &str = "channel-fuel-nightly.toml";

pub const DATE_FORMAT: &[FormatItem] = format_description!("[year]-[month]-[day]");
pub const DATE_FORMAT_URL_FRIENDLY: &[FormatItem] = format_description!("[year]/[month]/[day]");
