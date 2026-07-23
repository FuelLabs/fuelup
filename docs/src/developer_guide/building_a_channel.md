# Building a channel

`build-channel` is a Rust script that creates a [channel] that serves as a source of distribution of
the Fuel toolchain. This is accomplished by querying and collecting a list of downloadable components that
Fuel Labs publishes, creating a TOML file based on the collated data, which is then consumed by `fuelup`
during usage.

To learn about the args and options used in the script, skip to [Usage].

## Use cases

There are two main ways the `build-channel` script is used: in CI and manually.

### CI

The scheduled [`publish-nightly-channel.yml`] workflow builds the `nightly`
manifest at 01:00 UTC and publishes both the current manifest and a dated
archive when the build succeeds.

The [`update-channel.yml`] workflow is manually dispatched to update named
network manifests such as `mainnet` and `testnet`. It opens a pull request
against the `gh-pages` branch so the proposed component versions and hashes can
be reviewed before publication.

Fuelup resolves the `latest` runtime channel to the `mainnet` manifest; it is
not published by a separate "newest component" workflow.

### Manual

There may be times when we need a channel for a one-off event e.g. testnets. During these events, we do not

require a routine update, and can essentially publish once and be done. This is when manual publishing is done.

For example, a testnet manifest can be built locally like this:

```sh
# from fuelup project root
cargo run --locked -p build-channel -- \
  channel-fuel-testnet.toml YYYY-MM-DD \
  forc=<FORC_VERSION> fuel-core=<FUEL_CORE_VERSION>
```

Replace the date and version placeholders before running the command. Unlisted
components are resolved by `build-channel`; inspect every generated version,
download URL, and hash before publishing the manifest.

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

- _Optional_. A list of key-value pairs mapping component names to [SemVer]-compatible versions, e.g. 'fuel-core=0.17.1'

### Options

`--nightly`

- _Optional_. Specify if we are building a nightly channel.

[Usage]: #usage
[sway-nightly-binaries repository]: https://github.com/FuelLabs/sway-nightly-binaries
[`publish-nightly-channel.yml`]: https://github.com/FuelLabs/fuelup/blob/master/.github/workflows/publish-nightly-channel.yml
[`update-channel.yml`]: https://github.com/FuelLabs/fuelup/blob/master/.github/workflows/update-channel.yml
[channel]: ../concepts/channels.md
[variable]: https://docs.github.com/en/actions/learn-github-actions/variables
[SemVer]: https://semver.org/
