use fuelup::channel::CHANNELS;
use fuelup::constants::{CHANNEL_LATEST_FILE_NAME, CHANNEL_MAINNET_FILE_NAME};

#[test]
fn user_guides_list_every_public_channel() {
    let basics = include_str!("../docs/src/basics.md");
    let channels = include_str!("../docs/src/concepts/channels.md");

    for channel in CHANNELS {
        let formatted_channel = format!("`{channel}`");
        assert!(
            basics.contains(&formatted_channel),
            "basic usage guide is missing public channel {formatted_channel}"
        );
        assert!(
            channels.contains(&formatted_channel),
            "channels guide is missing public channel {formatted_channel}"
        );
    }
}

#[test]
fn user_guides_do_not_describe_latest_as_newest_upstream() {
    let basics = include_str!("../docs/src/basics.md");
    let channels = include_str!("../docs/src/concepts/channels.md");

    assert!(basics.contains("mainnet-compatible distribution"));
    assert!(channels.contains("same manifest as `mainnet`"));
}

#[test]
fn latest_channel_still_aliases_the_mainnet_manifest() {
    assert_eq!(
        CHANNEL_LATEST_FILE_NAME, CHANNEL_MAINNET_FILE_NAME,
        "the user guides describe `latest` as an alias of the mainnet manifest; \
         update basics.md, concepts/channels.md, concepts/toolchains.md, \
         developer_guide/building_a_channel.md and overrides.md if the alias \
         target changes"
    );
}
