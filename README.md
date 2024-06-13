# Fuelup: the Fuel toolchain manager

`fuelup` installs the Fuel toolchain from our official release channels, enabling you to easily keep the toolchain updated.

**To start using fuelup**, refer to our [Quickstart](https://github.com/FuelLabs/fuelup#quickstart) below.

## Quick Links

[The Fuelup Book](https://fuellabs.github.io/fuelup/master/)

[Developing fuelup itself](https://fuellabs.github.io/fuelup/master/developer_guide/index.html)

[Adding components to fuelup](https://fuellabs.github.io/fuelup/master/developer_guide/adding_components.html)

## Quickstart

Currently, this script supports Linux/macOS systems only. For other systems, please [read the Installation chapter](https://fuellabs.github.io/fuelup/master/installation/other.html).

Installation is simple: all you need is `fuelup-init.sh`, which downloads the core Fuel binaries needed to get you started on development.

```sh
curl -fsSL https://install.fuel.network/ | sh
```

This will automatically install `forc`, its accompanying plugins, `fuel-core` and other key components in `~/.fuelup/bin`. Please read the [Components chapter](https://fuellabs.github.io/fuelup/master/concepts/components.html) for more info on the components installed.

The script will ask for permission to add `~/.fuelup/bin` to your `PATH`.

Otherwise, you can also pass `--no-modify-path` so that `fuelup-init` does not modify your `PATH` and will not ask for permission to do so:

```sh
curl -fsSL https://install.fuel.network/ | sh -s -- --no-modify-path
```

If you just want `fuelup` without automatically installing the `latest` toolchain, you can pass the `--skip-toolchain-installation` option:

```sh
curl -fsSL https://install.fuel.network/ | sh -s -- --skip-toolchain-installation
```

## License

Apache License, Version 2.0, ([LICENSE](./LICENSE) or <https://www.apache.org/licenses/LICENSE-2.0>).
