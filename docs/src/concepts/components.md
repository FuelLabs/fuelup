# Components

Each [toolchain] has several "components", which are tools used to develop on Fuel.

The `fuelup component` command is used to manage the installed components.

Components can be added to a already-installed toolchain with the `fuelup component` command:

```sh
fuelup component add fuel-core
```

The following is an overview of the different components:

- [`forc`] — The Fuel Orchestrator, a suite of tools to work with the Fuel ecosystem.
- [`fuel-core`] — Full node implementation of the Fuel v2 protocol, written in Rust.
- [`forc-explore`] — A Forc plugin for running the Fuel Block Explorer.
- [`forc-fmt`] — A Forc plugin for running the Sway code formatter.
- [`forc-lsp`] - A Forc plugin for the Sway LSP (Language Server Protocol) implementation.

[toolchain]: toolchains.md
[`forc`]: https://fuellabs.github.io/sway/master/forc/index.html
[`fuel-core`]: https://github.com/FuelLabs/fuel-core
[`forc-explore`]: https://fuellabs.github.io/sway/master/forc_explore.html
[`forc-fmt`]: https://fuellabs.github.io/sway/master/forc_fmt.html
[`forc-lsp`]: https://fuellabs.github.io/sway/master/forc_lsp.html
