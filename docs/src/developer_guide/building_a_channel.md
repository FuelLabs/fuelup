# Building a channel

`build-channel` is a Rust script that creates a [channel] that serves as a source of distribution of
the Fuel toolchain. This is accomplished by querying and collecting a list of downloadable components that
Fuel Labs publishes, creating a TOML file based on the collated data, which is then consumed by `fuelup`
during usage.

To learn about the args and options used in the script, skip to [Usage].

## Use cases

There are 2 main ways the `build-channel` script is used: in the CI, and locally.

### CI

This script's main usage is found within the `fuelup` CI, where it publishes the channels to the [`gh-pages`] branch.

The `nightly` channel is published automatically. A channel is built at 01:00 UTC every day by the
[`Publish Channel (nightly)`] workflow, containing the download links to binaries found within the
[sway-nightly-binaries repository].

The `latest`, `mainnet` and `testnet` channels are published manually via the [`Update Channel`] workflow, which is
triggered with `workflow_dispatch`. An operator selects the channel and provides the component versions to pin; the
workflow runs `build-channel` and opens a pull request against `gh-pages`. (Note that the `latest` toolchain is served
by `channel-fuel-mainnet.toml`, so `latest` is updated by bumping the `mainnet` channel.) This flow is explained in more
detail in the [channels developer guide].

### Locally

Running `build-channel` locally is a good sanity check when working on this codebase, or when preparing the versions to
feed into the [`Update Channel`] workflow. It fails fast if a pinned version has missing release artifacts.

For example, building a `testnet` channel is done like so:

```sh
# from fuelup project root
cargo run --locked -p build-channel -- \
  channel-fuel-testnet.toml YYYY-MM-DD \
  forc=<FORC_VERSION> fuel-core=<FUEL_CORE_VERSION>
```

Replace the date and version placeholders before running the command. Unlisted
components are resolved by `build-channel`; inspect every generated version,
download URL, and hash before publishing the manifest.

## Usage

### Arguments

`OUT_FILE`

- Name of TOML file that will be created.

`PUBLISH_DATE`

- The date for when the channel was created and published.

`GITHUB_RUN_ID`

- _Optional_. This is the `$GITHUB_RUN_ID` [variable] in the GitHub CI. Identifies the specific run that a channel was published by.

`PACKAGES`

- _Optional_. A list of key-value pairs mapping component names to [SemVer]-compatible versions, e.g. 'fuel-core=0.17.1'

### Options

`--nightly`

- _Optional_. Specify if we are building a nightly channel.

[Usage]: #usage
[channels developer guide]: ../concepts/channels.md#developer-guide
[`gh-pages`]: https://github.com/FuelLabs/fuelup/tree/gh-pages
[`Update Channel`]: https://github.com/FuelLabs/fuelup/blob/master/.github/workflows/update-channel.yml
[`Publish Channel (nightly)`]: https://github.com/FuelLabs/fuelup/blob/master/.github/workflows/publish-nightly-channel.yml
[sway-nightly-binaries repository]: https://github.com/FuelLabs/sway-nightly-binaries
[channel]: ../concepts/channels.md
[variable]: https://docs.github.com/en/actions/learn-github-actions/variables
[SemVer]: https://semver.org/
