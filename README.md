# Fuelup: the Fuel toolchain manager

`fuelup` installs the Fuel toolchain from our official release channels, enabling you to easily keep the toolchain updated.

## Quickstart 

Currently, this script supports Linux/macOS systems only. For other systems, please [read the Installation chapter](https://fuellabs.github.io/fuelup/master/installation/other.html).

Installation is simple: all you need is `fuelup-init.sh`, which downloads the core Fuel binaries needed to get you started on development.

```sh
curl --proto '=https' --tlsv1.2 -sSf https://fuellabs.github.io/fuelup/fuelup-init.sh | sh
```

This will install `forc` and its accompanying plugins, as well as `fuel-core` in `~/.fuelup/bin`. Please read the [Components chapter](https://fuellabs.github.io/fuelup/master/concepts/components.html) for more info.

The script will ask for permission to add `~/.fuelup/bin` to your `PATH`.

Otherwise, you can also pass `--no-modify-path` so that `fuelup-init` does not modify your `PATH` and will not ask for permission to do so:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://fuellabs.github.io/fuelup/fuelup-init.sh | sh -s -- --no-modify-path
```

If you just want `fuelup` without automatically installing the `latest` toolchain, you can pass the `--skip-toolchain-installation` option:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://fuellabs.github.io/fuelup/fuelup-init.sh | sh -s -- --skip-toolchain-installation
```

## Book

For more details on how `fuelup` works, as well as usage examples, please refer to [The Fuelup Book](https://fuellabs.github.io/fuelup/master/).

## Contributing to Fuelup

We welcome contributions to fuelup!

Please see the [Contributing To Fuelup](https://fuellabs.github.io/fuelup/master/contributing_to_fuelup.html) of The Fuelup Book for guidelines and instructions to help you get started.

## License

Apache License, Version 2.0, ([LICENSE](./LICENSE) or <https://www.apache.org/licenses/LICENSE-2.0>)
