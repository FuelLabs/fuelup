# Contributing To Fuelup

Thanks for your interest in contributing to `fuelup`! This document outlines the process for installing and setting up `fuelup`, as well as some conventions on contributing to `fuelup`.

If you run into any difficulties getting started, you can always ask questions on our [Forum](https://forum.fuel.network/).

## Getting the repository

1. Visit the [fuelup](https://github.com/FuelLabs/fuelup) repo and fork the project.
2. Then clone your forked copy to your local machine and get to work.

```sh
git clone https://github.com/FuelLabs/fuelup
cd fuelup
```

## Building and testing

The following steps will run the fuelup test suite and ensure that everything is set up correctly.

First, run and ensure all tests pass:

```sh
cargo test
```

There are both unit tests and integration tests. Unit tests involve testing isolated components of the codebase,
while integration tests involve directly invoking the `fuelup` binary in a sandboxed environment with a
temporary filesystem.

Note that some integration tests involve installing a toolchain and adding components, which means they
will fail without internet connection.

Congratulations! You've now got everything setup and are ready to start making contributions.

## Finding something to work on

There are many ways in which you may contribute to `fuelup`, some of which involve coding knowledge and some which do not. A few examples include:

- Reporting bugs
- Adding documentation to the `fuelup` book
- Adding new features or bugfixes for which there is already an open issue
- Making feature requests

Check out our [Help Wanted](https://github.com/FuelLabs/fuelup/labels/help%20wanted), [Fuelup Book](https://github.com/FuelLabs/fuelup/labels/book) or [Good First Issue](https://github.com/FuelLabs/fuelup/labels/good%20first%20issue) issues to find a suitable task.

If you are planning something big, for example, related to multiple components or changes in current behaviors, make sure to open an issue to discuss with us before starting on the implementation.

## Contribution flow

This is a rough outline of what a contributor's workflow looks like:

- Make sure what you want to contribute is already tracked as an issue.
  - We may discuss the problem and solution to the issue.
- Create a Git branch from where you want to base your work. This is usually master.
- Write code, add test cases, and commit your work.
- Run tests and make sure all tests pass.
- If the PR contains any breaking changes, add the breaking label to your PR.
- Push your changes to a branch in your fork of the repository and submit a pull request.
  - Make sure to mention the issue, which is created in step 1, in the commit message.
- Your PR will be reviewed and some changes may be requested.
  - Once you've made changes, your PR must be re-reviewed and approved.
  - If the PR becomes out of date, you can use GitHub's 'update branch' button.
  - If there are conflicts, you can merge and resolve them locally. Then push to your PR branch.
    Any changes to the branch will require a re-review.
- Our CI system (Github Actions) automatically tests all authorized pull requests.
- Use Github to merge the PR once approved.

Thanks for your contributions!

### Linking issues

Pull requests should be linked to at least one issue in the same repo.

If the pull request resolves the relevant issues, and you want GitHub to close these issues automatically after it merged into the default branch, you can use the syntax (`KEYWORD #ISSUE-NUMBER`) like this:

```text
close #123
```

If the pull request links an issue but does not close it, you can use the keyword `ref` like this:

```text
ref #456
```

Multiple issues should use full syntax for each issue and be separated by a comma, like:

```text
close #123, ref #456
```
