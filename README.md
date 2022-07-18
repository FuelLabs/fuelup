# Fuelup: the Fuel toolchain installer

`fuelup` installs the Fuel toolchain from our official release channels, enabling you to easily keep the toolchain updated.

## Installation

Currently, this script supports Linux/macOS systems only. For other systems, please [install from source](https://fuellabs.github.io/sway/latest/introduction/installation.html#installing-from-source).

Installation is simple: all you need is `fuelup-init.sh`, which downloads the core Fuel binaries needed to get you started on development.

```sh
curl --proto '=https' --tlsv1.2 -sSf https://fuellabs.github.io/fuelup/fuelup-init.sh | sh
```

This will install `forc`, `forc-fmt`, `forc-explore`, `forc-lsp` as well as `fuel-core` in `~/.fuelup/bin`. The script will ask for permission to add `~/.fuelup/bin` to your `PATH`.

Otherwise, you can also pass `--no-modify-path` so that `fuelup-init` does not modify your `PATH` and will not ask for permission to do so:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://fuellabs.github.io/fuelup/fuelup-init.sh | sh -s -- --no-modify-path
```

In future, `fuelup` will also let you switch between toolchains, allowing for a smooth developer experience while allowing you to have more flexibility, along with other features.

## Usage

To install and update the latest toolchain:

```sh
fuelup toolchain install latest
```

To update `fuelup` itself:

```sh
fuelup self update
```

To check for updates:

```sh
fuelup check
```

## Channels

`fuelup` uses channels to distribute Fuel toolchain binaries. Click [here](https://github.com/FuelLabs/fuelup/blob/master/CHANNELS.md) to read more about channels.

## License

Apache License, Version 2.0, ([LICENSE](./LICENSE) or <https://www.apache.org/licenses/LICENSE-2.0>)
