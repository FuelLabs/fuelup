# Building a channel

`build-channel` is a Rust script that creates a [channel] that serves as a source of distribution of
the Fuel toolchain. This is accomplished by querying and collecting a list of downloadable components that
Fuel Labs publishes, creating a TOML file based on the collated data, which is then consumed by `fuelup`
during usage.

To learn about the args and options used in the script, skip to [Usage].

## Use cases

There are 2 main ways where the `build-channel` script is used: in the CI, and manually.

### CI

This script's main usage is found within the `fuelup` CI. This script is in charge of publishing the `latest` and
`nightly` channels on a routine basis.

The `latest` channel is re-built if the [check versions workflow] detects a new release of `forc` or `fuel-core`, and
compatibility tests pass after that. This is explained in detail in the [latest channel developer guide].

An example of this usage is in [test-toolchain-compatibility.yml].

The `nightly` channel is more straightforward - a channel is built at 01:00 UTC every day, containing the download
links to binaries found within the [sway-nightly-binaries repository].

An example of this usage is in [publish-nightly-channel.yml].

### Manual

There may be times where we need a channel for a one-off event eg. testnets. During these events, we do not
require a routine update, and can essentially publish once and be done. This is when manual publishing is done.

For example, building a `beta-3` toolchain is done like so:

```sh
# from fuelup project root
cd ci/build-channel && cargo run -- channel-fuel-beta-3.toml 2023-02-13 forc=0.35.0 fuel-core=0.17.1
```

The above command means that we're building a channel named `channel-fuel-beta-3.toml` with the date `2023-02-13` (YYYY-MM-DD)
and `forc` and `fuel-core` versions `0.35.0` and `0.17.1` respectively, and the latest versions for the other unlisted components.

Other than for these one-off events, manually running `build-channel` locally is a good sanity check when working
on this codebase.

## Usage

### Arguments

`OUT_FILE`

- Name of TOML file that will be created.

`PUBLISH_DATE`

- The date for when the channel was created and published.

`GITHUB_RUN_ID`

- _Optional_. This is the `$GITHUB_RUN_ID` [variable] in the GitHub CI. Identifies the specific run that a channel was published by.

`PACKAGES`

- _Optional_. A list of key-value pairs mapping component names to [SemVer]-compatible versions, eg. 'fuel-core=0.17.1'

### Options

`--nightly`

- _Optional_. Specify if we are building a nightly channel.

[Usage]: #usage
[check versions workflow]: https://github.com/FuelLabs/fuelup/actions/workflows/index-versions.yml
[latest channel developer guide]: ../concepts/channels/latest.html#understanding-the-latest-workflow
[test-toolchain-compatibility.yml]: https://github.com/FuelLabs/fuelup/blob/3abe817673184ac17a78b2a8965234813ac6d911/.github/workflows/test-toolchain-compatibility.yml#L174
[sway-nightly-binaries repository]: https://github.com/FuelLabs/sway-nightly-binaries 
[publish-nightly-channel.yml]: https://github.com/FuelLabs/fuelup/blob/3abe817673184ac17a78b2a8965234813ac6d911/.github/workflows/publish-nightly-channel.yml#L37
[channel]: ../concepts/channels/index.md
[variable]: https://docs.github.com/en/actions/learn-github-actions/variables
[SemVer]: https://semver.org/
