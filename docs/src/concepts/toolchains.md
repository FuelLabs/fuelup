# Toolchains

<!-- This section should give a basic overview of toolchains -->
<!-- toolchains:example:start -->
Many `fuelup` commands deal with _toolchains_, a single installation of the
Fuel toolchain. `fuelup` supports **two** types of toolchains.

1. Distributable toolchains which track the official release [channels]
   (`latest`, `mainnet`, `testnet`, and `nightly`);
2. Custom toolchains, in which individual components can be installed in a
   modular manner.
<!-- toolchains:example:end -->

[channels]: channels.md

## Toolchain specification

Distributable toolchain names have the following form:

```text
<channel>[-<date>][-<host>]

<channel>       = latest | nightly | mainnet | testnet
<date>          = YYYY-MM-DD
<host>          = <target-triple>
```

`channel` is a named release channel. `latest` and `nightly` can be appended
with an archive date, as in `nightly-2025-01-18`, in which case the toolchain
is downloaded from the archive for that date. A dated name works only when its
matching manifest was actually published; not every date is available.

`mainnet` and `testnet` are named, undated network channels.

Finally, the host may be specified as a target triple.

## Custom toolchains

For most use cases, using the officially packaged toolchains is good enough.

For advanced use cases, `fuelup` allows you to build a toolchain in a
modular manner, and to specify specific versions of components to install.

To initialize a new, empty toolchain:

```sh
fuelup toolchain new my_toolchain
```

Now you can add/remove components to/from the toolchain as you wish:

```sh
fuelup component add forc
```

In custom toolchains, you can specify a specific version of a component to install:

```sh
fuelup component add forc@0.19.2
```
