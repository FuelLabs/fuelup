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

    // The marker phrases above cannot catch a contradictory claim added
    // elsewhere, so also require the explicit negations to survive and allow
    // "newest upstream" wording only inside a negating or qualifying context.
    let latest_section = channels
        .split("<!-- latest:example:start -->")
        .nth(1)
        .and_then(|section| section.split("<!-- latest:example:end -->").next())
        .expect("channels guide must keep the latest section markers");
    assert!(
        latest_section.contains("alias for the `mainnet` channel"),
        "the latest section must describe `latest` as a mainnet alias"
    );
    assert!(
        latest_section.contains("does not mean the newest upstream"),
        "the latest section must keep the newest-upstream negation"
    );

    let basics_normalized = basics.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(
        basics_normalized.contains("does not mean the newest upstream release"),
        "the basics guide must keep the newest-upstream negation"
    );

    for (name, guide) in [("basics.md", basics), ("channels.md", channels)] {
        let normalized = guide
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase();
        let needle = "newest upstream";
        let mut from = 0;
        while let Some(pos) = normalized[from..].find(needle) {
            let hit = from + pos;
            let mut window_start = hit.saturating_sub(40);
            while !normalized.is_char_boundary(window_start) {
                window_start -= 1;
            }
            let window = &normalized[window_start..hit];
            assert!(
                window.contains("not mean the") || window.contains("behind the"),
                "{name} mentions 'newest upstream' outside a negation or \
                 qualifier: ...{window}{needle}..."
            );
            from = hit + needle.len();
        }
    }
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
