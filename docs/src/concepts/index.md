# Concepts

This section will explain how fuelup works on a technical level and explains each
component of fuelup.

## Terminology

<!-- This section should explain fuelup terminology -->
<!-- terms:example:start -->
- **channel** — The Fuel toolchain is released to different "channels".
  Currently, the **latest**, **nightly**, **testnet** and **mainnet** channels
  are published. See the [Channels] chapter for more details.

- **toolchain** — A "toolchain" is an installation of the
  Fuel Orchestrator (`forc`), its related plugins (like `forc-fmt`) and
  the Fuel client (`fuel-core`). A [toolchain specification] includes the
  release channel and the host platform that the toolchain runs on.

  A toolchain can be installed either through the channels, or be modularly
  constructed as a [custom toolchain].

- **component** — Each release of the Fuel toolchain includes several "components",
  which are tools used to develop on Fuel. See the [Components] chapter for more details.
<!-- terms:example:end -->

[components]: components.md
[custom toolchain]: toolchains.md#custom-toolchains
[toolchain specification]: toolchains.md
[channels]: channels.md
