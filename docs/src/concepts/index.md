# Concepts

This section will explain how fuelup works on a technical level and explains each
component of fuelup.

## Terminology

- **channel** — The Fuel toolchain will be released to different "channels".
  Currently, it is only released to the **latest** channel.
  See the [Channels] chapter for more details.

- **toolchain** — A "toolchain" is an installation of the
  Fuel Orchestrator (`forc`), its related plugins (like `forc-fmt`) and
  the Fuel client (`fuel-core`). A [toolchain specification] includes the
  release channel and the host platform that the toolchain runs on.

  A toolchain can be installed either through the channels, or be modularly
  constructed as a [custom toolchain].

- **component** — Each release of the Fuel toolchain includes several "components",
  which are tools used to develop on Fuel. See the [Components] chapter for more details.

[components]: components.md
[custom toolchain]: toolchains.md#custom-toolchains
[toolchain specification]: toolchains.md
[channels]: channels.md
