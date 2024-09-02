use time::{format_description::FormatItem, macros::format_description};

// Channels

pub const LATEST: &str = "latest";
pub const NIGHTLY: &str = "nightly";
pub const BETA_1: &str = "beta-1";
pub const BETA_2: &str = "beta-2";
pub const BETA_3: &str = "beta-3";
pub const BETA_4: &str = "beta-4";
pub const BETA_5: &str = "beta-5";
pub const DEVNET: &str = "devnet";
pub const TESTNET: &str = "testnet";
// Stable is reserved, although currently unused.
pub const STABLE: &str = "stable";

pub const BETA_CHANNELS: [&str; 7] = [BETA_1, BETA_2, BETA_3, BETA_4, BETA_5, DEVNET, TESTNET];

#[allow(clippy::indexing_slicing)]
pub const fn generate_all_channels<const N: usize>() -> [&'static str; N] {
    let mut channels = [""; N];

    channels[0] = LATEST;
    channels[1] = NIGHTLY;

    let mut i = 0;

    while i < BETA_CHANNELS.len() {
        channels[2 + i] = BETA_CHANNELS[i];
        i += 1;
    }

    channels
}

pub const CHANNELS: [&str; 9] = generate_all_channels::<9>();

// URLs

pub const GITHUB_API_ORG_URL: &str = "https://api.github.com/repos/FuelLabs/";
pub const GITHUB_USER_CONTENT_URL: &str = "https://raw.githubusercontent.com/FuelLabs/";

// NOTE: Although this variable is named "LATEST", it needs to point to
// "testnet" until mainnet has launched. Once launched, we can then merge this
// variable with the `CHANNEL_LATEST_URL_ACTUAL` variable
pub const CHANNEL_LATEST_URL: &str =
    "https://raw.githubusercontent.com/FuelLabs/fuelup/gh-pages/channel-fuel-testnet.toml";

// We need to point to the latest URL but we can't use `CHANNEL_LATEST_URL`
// until mainnet has launched (as `CHANNEL_LATEST_URL` currently points to
// testnet). So we dupilcate here for now and we can cleanup sometime later
pub const CHANNEL_LATEST_URL_ACTUAL: &str =
    "https://raw.githubusercontent.com/FuelLabs/fuelup/gh-pages/channel-fuel-latest.toml";

// Filenames

pub const FUEL_TOOLCHAIN_TOML_FILE: &str = "fuel-toolchain.toml";
pub const FUELS_VERSION_FILE: &str = "fuels_version";

pub const CHANNEL_LATEST_FILE_NAME: &str = "channel-fuel-testnet.toml";
pub const CHANNEL_NIGHTLY_FILE_NAME: &str = "channel-fuel-nightly.toml";
pub const CHANNEL_BETA_1_FILE_NAME: &str = "channel-fuel-beta-1.toml";
pub const CHANNEL_BETA_2_FILE_NAME: &str = "channel-fuel-beta-2.toml";
pub const CHANNEL_BETA_3_FILE_NAME: &str = "channel-fuel-beta-3.toml";
pub const CHANNEL_BETA_4_FILE_NAME: &str = "channel-fuel-beta-4.toml";
pub const CHANNEL_BETA_5_FILE_NAME: &str = "channel-fuel-beta-5.toml";
pub const CHANNEL_DEVNET_FILE_NAME: &str = "channel-fuel-devnet.toml";
pub const CHANNEL_TESTNET_FILE_NAME: &str = "channel-fuel-testnet.toml";

// Misc

pub const DATE_FORMAT: &[FormatItem] = format_description!("[year]-[month]-[day]");
pub const DATE_FORMAT_URL_FRIENDLY: &[FormatItem] = format_description!("[year]/[month]/[day]");
