# fuelup: the Fuel toolchain installer

_fuelup_ installs the Fuel toolchain from our official release channels, enabling you to easily keep the toolchain updated.

## Installation

Installation is simple: all you need is `fuelup-init.sh`, which downloads the core Fuel binaries needed to get you started on development.

```sh
curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/FuelLabs/fuelup/master/fuelup-init.sh | sh -s install
```

This will install `forc`, `forc-fmt`, `forc-explore`, `forc-lsp` as well as `fuel-core` in `~/.fuelup/bin`. You will have to add `~/.fuelup/bin` to your `PATH`.

In future, _fuelup_ will also let you switch between toolchains, allowing for a smooth developer experience while allowing you to have more flexibility, along with other features.

## License

Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)



