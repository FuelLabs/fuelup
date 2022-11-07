# Components

Each [toolchain] has several "components", which are tools used to develop on Fuel.

The `fuelup component` command is used to manage the installed components.

Components can be added to an already-installed toolchain with the `fuelup component` command:

```sh
fuelup component add forc
```

In custom toolchains, you also have the choice of adding a specific version of a component:

```sh
fuelup component add forc@0.30.1
```

## Components overview

The following is an overview of components installable through `fuelup`:

- [`forc`] — The Fuel Orchestrator, a suite of tools to work with the Fuel ecosystem. This comes
with some built-in plugin executables, namely [`forc-client`], [`forc-fmt`] and [`forc-lsp`].
- [`fuel-core`] — Full node implementation of the Fuel v2 protocol, written in Rust.
- [`forc-explore`] — A Forc plugin for running the Fuel Block Explorer.
- [`forc-wallet`] - A Forc plugin for managing Fuel wallets.

[toolchain]: toolchains.md
[`forc`]: https://fuellabs.github.io/sway/master/forc/index.html
[`fuel-core`]: https://github.com/FuelLabs/fuel-core
[`forc-explore`]: https://fuellabs.github.io/sway/master/forc/plugins/forc_explore.html
[`forc-fmt`]: https://fuellabs.github.io/sway/master/forc/plugins/forc_fmt.html
[`forc-lsp`]: https://fuellabs.github.io/sway/master/forc/plugins/forc_lsp.html
[`forc-client`]: https://fuellabs.github.io/sway/master/forc/plugins/forc_client/index.html
[`forc-wallet`]: https://github.com/FuelLabs/forc-wallet
