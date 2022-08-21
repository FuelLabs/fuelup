# Installation

`fuelup` installs `forc` and `fuel-core`, and other plugins like
`forc-fmt`, `forc-lsp` and `forc-explore` to Fuelup's `bin` directory.
On Unix it is located at `$HOME/.fuelup/bin`.

This directory can automatically be in your `PATH` environment variable if
allowed in the installation step (explained below), which means you can run them from the shell without further configuration.

## Quickstart

Installation is done through the `fuelup-init` script found on our [repository], where you may find the source code.

Run the following command:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://fuellabs.github.io/fuelup/fuelup-init.sh | sh
```

This will install `forc`, `forc-fmt`, `forc-explore`, `forc-lsp` as well as `fuel-core` in `~/.fuelup/bin`. The script will ask for permission to add `~/.fuelup/bin` to your `PATH`.

Otherwise, you can also pass `--no-modify-path` so that `fuelup-init` does not modify your `PATH` and will not ask for permission to do so:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://fuellabs.github.io/fuelup/fuelup-init.sh | sh -s -- --no-modify-path
```

Ensure that all components are downloaded and works:

```sh
fuelup --version; forc --version; fuel-core --version; forc-fmt --version; forc-lsp --version; forc-explore --version
```

[repository]: https://github.com/FuelLabs/fuelup
