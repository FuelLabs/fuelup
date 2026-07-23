# Basic usage

The quickest way to get started on Fuel mainnet is to install the `latest`
toolchain. This step is normally performed automatically when `fuelup` is
installed through `fuelup-init`:

```sh
fuelup toolchain install latest
```

`latest` is the default alias for the mainnet-compatible distribution. It does
not mean the newest upstream release of every component.

## Choosing a channel

Fuelup publishes four channels:

| Channel | Intended use |
| ------- | ------------ |
| `latest` | Default alias for the mainnet-compatible distribution |
| `mainnet` | Tooling selected for Fuel mainnet |
| `testnet` | Tooling selected for Fuel testnet |
| `nightly` | Daily development builds; compatibility is not guaranteed |

Install the named channel for the network you are targeting:

```sh
# Fuel mainnet
fuelup toolchain install mainnet

# Fuel testnet
fuelup toolchain install testnet

# Unreleased development builds
fuelup toolchain install nightly
```

See [release channels] for the source, release policy, and stability guarantees
of each channel.

## Keeping installed Fuel toolchains up to date

Run `fuelup update` to refresh every distributable channel that is already
installed. This does not change which channel is the default.

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

## Using Http Proxy

To configure `fuelup` to use your proxy setting you can change `http_proxy`(***other optional environments see below***) environment value. The value format is in [libcurl format](https://everything.curl.dev/usingcurl/proxies/type.html) as in `[protocol://]host[:port]`.

### Supported proxy environment variables

- http_proxy
- HTTP_PROXY
- https_proxy
- HTTPS_PROXY
- all_proxy
- ALL_PROXY

***Warning: don't leave all proxy environment with empty string or other invalid format***

## Help system

The `fuelup` command-line is built with [clap], which serves a nice, built-in help system
that provides more information about each command. Run `fuelup help` for an overview. Detailed
help for each subcommand is also available.

For example, run `fuelup component --help` for specifics on installing [components].

[release channels]: concepts/channels.md
[clap]: https://github.com/clap-rs/clap
[components]: concepts/components.md
