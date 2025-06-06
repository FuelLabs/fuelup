# Installation

<!-- This section should explain what fuelup installs -->
<!-- fuelup:example:start -->
`fuelup` installs `forc` and `fuel-core`, and other plugins like
`forc-client` and `forc-fmt` to Fuelup's `bin` directory.
On Unix it is located at `$HOME/.fuelup/bin`.
<!-- fuelup:example:end -->

This directory can automatically be in your `PATH` environment variable if
allowed in the installation step (explained below), which means you can run them from the shell without further configuration.

## Quickstart

Installation is done through the `fuelup-init` script found on our [repository], where you may find the source code.

Run the following command (you may need to set a [http proxy](../basics.md#using-http-proxy) before running this command):

<!-- This section should have the default command to install fuelup -->
<!-- install:example:start -->
```sh
curl -fsSL https://install.fuel.network/ | sh
```
<!-- install:example:end -->

This will install `forc`, `forc-client`, `forc-fmt`, `forc-crypto`, `forc-call`, `forc-debug`, `forc-migrate`, `forc-lsp`, `forc-node`, `forc-publish`, `forc-wallet` as well as `fuel-core` in `~/.fuelup/bin`. The script will ask for permission to add `~/.fuelup/bin` to your `PATH`.

Otherwise, you can also pass `--no-modify-path` so that `fuelup-init` does not modify your `PATH` and will not ask for permission to do so:

```sh
curl -fsSL https://install.fuel.network/ | sh -s -- --no-modify-path
```

Ensure that all components are downloaded and works:

```sh
fuelup --version
forc --version
fuel-core --version
forc-deploy --version
forc-fmt --version
forc-crypto --version
forc-call --version
forc-debug --version
forc-lsp --version
forc-migrate --version
forc-node --version
forc-publish --version
forc-run --version
forc-submit --version
```

[repository]: https://github.com/FuelLabs/fuelup
