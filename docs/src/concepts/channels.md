# Channels

<!-- This section should give an overview of fuelup channels -->
<!-- channels:example:start -->
`fuelup` adopts a simplified version of `rustup` [channels](https://rust-lang.github.io/rustup/concepts/channels.html). Currently, the `latest`, `nightly`, `testnet`, and `mainnet` channels are published and serve as a source of distribution of Fuel toolchain binaries.

| Channel       | Source          | Integration Tested   | Update Frequency         | Available |
| ------------- | --------------- | -------------------- | ------------------------ | --------- |
| **[latest]**  | published bins  | ✔️                    | only when necessary      | ✔️         |
| **[nightly]** | `master` branch | ➖                   | nightly (1:00 AM UTC)    | ✔️         |
| **[mainnet]** | published bins  | ➖                   | only when necessary      | ✔️         |
| **[testnet]** | published bins  | ➖                   | only when necessary      | ✔️         |
<!-- channels:example:end -->

## The `latest` channel

<!-- This section should give an overview of the latest channel -->
<!-- latest:example:start -->
The `latest` channel is pointing to our mainnet network. This toolchain should be used to interact with and build on mainnet. This is also the default channel for `fuelup`.
<!-- latest:example:end -->

## The `nightly` channel

<!-- This section should give an overview of the nightly channel -->
<!-- nightly:example:start -->
The `nightly` channel is a published TOML file describing successful builds of the `master` branch of `forc` and `fuel-core` for the day.
These builds are released in the [sway-nightly-binaries] repository and the workflows in that repo start building them every day at **00:00 UTC**.

The `nightly` channel within `fuelup` is updated by a scheduled GitHub workflow that **runs every day at 01:00 UTC**, after builds have finished.
Note that the `nightly` channel might fail to build, in which case it is possible that the `nightly` toolchain may not be available for that day.

You should use `nightly` if you want the latest changes to `master` that have not been officially released yet.
Keep in mind that compatibility between `forc` and `fuel-core` is not guaranteed here, and you should expect unstable features to break.
<!-- nightly:example:end -->

## The `mainnet` channel

The `mainnet` channel is a published TOML file describing the toolchain that is compatible with our final public [mainnet](https://fuel.mirror.xyz/U2xqey0oZFCrIBf7bO3yYyFplJKiJ0sfzAyYmhrPjag) on the Ignition network. This toolchain should be used to interact with and build on the mainnet. The components to be installed can be found [here](https://github.com/FuelLabs/fuelup/blob/gh-pages/channel-fuel-mainnet.toml).

## The `testnet` channel

The `testnet` channel is a published TOML file describing the toolchain that is compatible with our final public [testnet](https://fuel.mirror.xyz/fztu8P_NJz6k7qLXhaGHX3ArctlQpxsUp87pgRpJ4bE) on the Sepolia network. This toolchain should be used to interact with and build on the testnet. The components to be installed can be found [here](https://github.com/FuelLabs/fuelup/blob/gh-pages/channel-fuel-testnet.toml).

## Developer Guide

### Understanding the `latest` workflow

> **Note**
>
> Reading the information below is only really necessary if you wish to contribute to the workflows or want a deeper understanding on how channels are updated.

The entry point of the scheduled workflow is within `index-versions.yml`. We run the Rust script `compare-versions` to collect versions of `forc` and `fuel-core` to be tested. These versions are filtered for incompatible versions prior to being used as a JSON string input to `test-toolchain-compatibility.yml`, where the testing occurs.

In `test-toolchain-compatibility.yml`, The versions JSON string input is used to initialize a matrix using the [`fromJSON`](https://docs.github.com/en/actions/learn-github-actions/expressions#fromjson) expression. We checkout the Sway repo at the given `forc` version and pull the `fuel-core` Docker image at the given `fuel-core` version and run integration tests found in the [Sway CI](https://github.com/FuelLabs/sway/blob/3bd8eaf4a0f11a3009c9421100cc06c2e897b6c2/.github/workflows/ci.yml#L229-L270) for them.

Note that we only mark versions as incompatible specifically if tests fail, and not if other prior steps fail (e.g. we do not want to mark versions as incompatible if there were errors pulling the Docker image)

The [upload-artifact](https://github.com/actions/upload-artifact) action is used to collect the test results from the matrix to be used later in a [download-artifact](https://github.com/actions/download-artifact) step.

If tests were not skipped and are now done, we finally get to the `index-versions` job. We download the artifacts that were previously uploaded to be used here. This job will:

1. upload incompatible versions to `gh-pages`. These incompatible versions are named in the format `incompatible-forc-<FORC_VERSION>@fuel-core-<FUEL_CORE_VERSION>`.

2. update the channel by filtering for the latest versions of `forc` and `fuel-core` that passed tests within the matrix by running `index-versions.sh`. These are named in the format `compatible-forc-<FORC_VERSION>@fuel-core-<FUEL_CORE_VERSION>`. Note that these files are not saved or uploaded onto `gh-pages` - they are only a way for the `test-toolchain-compatibility` job to share test results with this job.

### Debugging the workflow

If you're contributing to the workflows, it might be a good idea to fork the repo and test any changes you've made on a personal repo first.

Some changes you might want to make to allow for easier testing:

1. You may want to use the [push](https://docs.github.com/en/actions/using-workflows/triggering-a-workflow#using-a-single-event) or [workflow_dispatch](https://docs.github.com/en/actions/using-workflows/triggering-a-workflow#defining-inputs-for-manually-triggered-workflows) triggers to make testing easier.

2. You can also exit with 0 or 1 in jobs or steps where you want it to pass/fail.

You may also use [`nektos/act`](https://github.com/nektos/act) to run the workflow(s) locally.

[sway-nightly-binaries]: https://github.com/FuelLabs/sway-nightly-binaries/releases
[latest]: #the-latest-channel
[nightly]: #the-nightly-channel
[testnet]: #the-testnet-channel
