# Channels

`fuelup` adopts a simplified version of `rustup` [channels](https://rust-lang.github.io/rustup/concepts/channels.html). Currently, only the `latest` channel will be published and serve as a source of distribution of Fuel toolchain binaries.

The `latest` channel is updated by a scheduled GitHub workflow that **runs every 30 minutes** and checks for new, compatible releases of `forc` and `fuel-core`.

### `latest`

The `latest` channel is `fuelup`'s default channel. It provides access to the latest compatible, published releases of `forc` and `fuel-core`.

When installing the `latest` channel, fuelup will refer to the `channel-fuel-latest.toml` file within this repository to determine the set of `forc` and `fuel-core` versions to retrieve. The versions in this file are updated by a scheduled GitHub workflow that runs once every 30 minutes and performs the following steps:

1. Checks for newly published versions of forc and fuel-core.
2. Tests compatibility of new versions against a set of integration tests.
3. Selects the latest set of versions that successfully pass the tests.
4. Publishes the selected versions to the channel-fuel-latest.toml manifest.

## Developer Guide

### Understanding the `latest` workflow

_Note: Reading the information below is only really necessary if you wish to contribute to the workflows or want a deeper understanding on how channels are updated_

The entrypoint of the scheduled workflow is within `index-versions.yml`. We run the Rust script `compare-versions` to collect versions of `forc` and `fuel-core` to be tested. These versions are filtered for incompatible versions prior to being used as a JSON string input to `test-toolchain-compatibility.yml`, where the testing occurs.

In `test-toolchain-compatibility.yml`, The versions JSON string input is used to init a matrix using the [fromJSON](https://docs.github.com/en/actions/learn-github-actions/expressions#fromjson) expression. We checkout the Sway repo at the given `forc` version and pull the `fuel-core` Docker image at the given `fuel-core` version and run integration tests found in the [Sway CI](https://github.com/FuelLabs/sway/blob/3bd8eaf4a0f11a3009c9421100cc06c2e897b6c2/.github/workflows/ci.yml#L229-L270) for them.

Note that we only mark versions as incompatible specifically if tests fail, and not if other prior steps fail (eg. we do not want to mark versions as incompatible if there were errors pulling the Docker image)

The [upload-artifact](https://github.com/actions/upload-artifact) action is used to collect the test results from the matrix to be used later in a [download-artifact](https://github.com/actions/download-artifact) step.

If tests were not skipped and are now done, we finally get to the `index-versions` job. We download the artifacts that were previously uploaded to be used here. This job will:

1. upload incompatible versions to gh-pages. These incompatible versions are named in the format `incompatible-forc-<FORC_VERSION>@fuel-core-<FUEL_CORE_VERSION>`.

2. update the channel by filtering for the latest versions of `forc` and `fuel-core` that passed tests within the matrix by running `index-versions.sh`. These are named in the format `compatible-forc-<FORC_VERSION>@fuel-core-<FUEL_CORE_VERSION>`. Note that these files are not saved or uploaded onto gh-pages - they are only a way for the `test-toolchain-compatibility` job to share test results with this job.

### Debugging the workflow

If you're contributing to the workflows, it might be a good idea to fork the repo and test any changes you've made on a personal repo first.

Some changes you might want to make to allow for easier testing:

1. You may want to use the [push](https://docs.github.com/en/actions/using-workflows/triggering-a-workflow#using-a-single-event) or [workflow_dispatch](https://docs.github.com/en/actions/using-workflows/triggering-a-workflow#defining-inputs-for-manually-triggered-workflows) triggers to make testing easier.

2. You can also exit with 0 or 1 in jobs or steps where you want it to pass/fail.

You may also use [nektos/act](https://github.com/nektos/act) to run the workflow(s) locally.
