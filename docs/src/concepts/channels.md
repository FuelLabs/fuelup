# Channels

<!-- This section should give an overview of fuelup channels -->
<!-- channels:example:start -->
`fuelup` adopts a simplified version of `rustup` [channels](https://rust-lang.github.io/rustup/concepts/channels.html). Currently, `latest`, `mainnet`, `testnet`, and `nightly` are the public distribution channels.

| Channel | Source | Intended use | Update frequency |
| ------- | ------ | ------------ | ---------------- |
| **[latest]** | Alias of the `mainnet` manifest | Default/mainnet | With `mainnet` |
| **[mainnet]** | Published manifest | Fuel mainnet | When the supported mainnet toolchain changes |
| **[testnet]** | Published manifest | Fuel testnet | When the supported testnet toolchain changes |
| **[nightly]** | Development builds | Testing unreleased changes | Daily when builds succeed |
<!-- channels:example:end -->

## The `latest` channel

<!-- This section should give an overview of the latest channel -->
<!-- latest:example:start -->
Undated `latest` is the default channel and resolves to the same manifest as `mainnet`. Dated `latest-YYYY-MM-DD` names refer to separately archived manifests, which are not published for every date. Use the undated channel to interact with and build for mainnet. The name means "the default mainnet-compatible toolchain"; it does not mean the newest upstream Sway, Forc, Fuel Core, or plugin release.
<!-- latest:example:end -->

## The `nightly` channel

<!-- This section should give an overview of the nightly channel -->
<!-- nightly:example:start -->
The `nightly` channel is a published TOML file describing development builds of Forc and Fuel Core for the day.
These builds are released in the [sway-nightly-binaries] repository, whose workflows start building them every day at **00:00 UTC**.

The `nightly` channel within `fuelup` is updated by a scheduled GitHub workflow that **runs every day at 01:00 UTC**, after builds have finished.
Note that the `nightly` channel might fail to build, in which case it is possible that the `nightly` toolchain may not be available for that day.

You should use `nightly` if you want the latest changes to `master` that have not been officially released yet.
Keep in mind that compatibility between `forc` and `fuel-core` is not guaranteed here, and you should expect unstable features to break.
<!-- nightly:example:end -->

## The `mainnet` channel

The `mainnet` channel is a published TOML file describing the toolchain selected for Fuel mainnet on the Ignition network. Use this toolchain to interact with and build for mainnet. Its components and artifact hashes are in the [mainnet manifest].

## The `testnet` channel

The `testnet` channel is a published TOML file describing the toolchain selected for Fuel testnet on the Sepolia network. Use this toolchain to interact with and build for testnet. Its components and artifact hashes are in the [testnet manifest].

The version numbers in a network channel may be behind the newest upstream component releases. This is intentional: choose the channel for the network you need to target, not by comparing its name with upstream release numbers.

See [Building a channel] for the current publishing workflow.

[sway-nightly-binaries]: https://github.com/FuelLabs/sway-nightly-binaries/releases
[mainnet manifest]: https://github.com/FuelLabs/fuelup/blob/gh-pages/channel-fuel-mainnet.toml
[testnet manifest]: https://github.com/FuelLabs/fuelup/blob/gh-pages/channel-fuel-testnet.toml
[Building a channel]: ../developer_guide/building_a_channel.md
[latest]: #the-latest-channel
[nightly]: #the-nightly-channel
[testnet]: #the-testnet-channel
[mainnet]: #the-mainnet-channel
