# Basic usage

The quickest way to get started is to install the `latest` toolchain, although this step should be automatically done if you
installed `fuelup` via `fuelup-init`:

```sh
fuelup toolchain install latest
```

## Keeping the Fuel toolchain up to date

The Fuel toolchain is distributed on one [release channel]: latest (with nightly being a WIP).
`fuelup` uses the `latest` channel by default, which represents the latest stable release of the Fuel toolchain.

When new versions of the components within an distributable Fuel toolchain (`latest` or `nightly`)
are released, simply type `fuelup update` to update:

<!-- This section should show the command to update distributable toolchains -->
<!-- update:example:start -->
```sh
fuelup update
```
<!-- update:example:end -->

## Keeping `fuelup` up to date

You can request that `fuelup` update itself to the latest version of `fuelup`
by running:

<!-- This section should show the command to update fuelup -->
<!-- update_fuelup:example:start -->
```sh
fuelup self update
```
<!-- update_fuelup:example:end -->

## Help system

The `fuelup` command-line is built with [clap], which serves a nice, built-in help system
that provides more information about each command. Run `fuelup help` for an overview. Detailed
help for each subcommand is also available.

For example, run `fuelup component --help` for specifics on installing [components].

[release channel]: concepts/channels.md
[clap]: https://github.com/clap-rs/clap
[components]: concepts/components.md
