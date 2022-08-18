# Concepts

## How fuelup works

`fuelup` is largely inspired by [`rustup`]. It installs and manages multiple Fuel
toolchains and presents them all through a single set of tools installed to
`~/.fuelup/bin`. The [`forc`] and [`fuel-core`] executables installed in
`~/.fuelup/bin` are _[proxies]_ that delegate to the real toolchain. `fuelup`
then provides mechanisms to easily change the active toolchain by
reconfiguring the behavior of the proxies.

When `fuelup-init` is first executed, `fuelup` automatically installs the
`latest` toolchain. Proxies are created in `$HOME/.fuelup/bin`, while toolchains
are installed within `$HOME/.fuelup/toolchains`, in their own directories.

Running `forc` on the `latest` toolchain, for example, runs the proxy, which
then executes the appropriate `forc` found in the `latest` toolchain directory.

[`rustup`]: https://github.com/rust-lang/rustup
[`forc`]: https://fuellabs.github.io/sway/master/forc/index.html
[`fuel-core`]: https://github.com/FuelLabs/fuel-core
[proxies]: proxies.md

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
[profiles]: profiles.md
[toolchain specification]: toolchains.md
[channels]: channels.md
