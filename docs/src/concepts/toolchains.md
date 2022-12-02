# Toolchains

Many `fuelup` commands deal with _toolchains_, a single installation of the
Fuel toolchain. `fuelup` supports **two** types of toolchains.

1. Distributable toolchains which track the official release [channels] (eg, _latest_, _nightly_);
2. Custom toolchains and install individual components in a modular manner.

[channels]: channels/index.md

## Toolchain specification

Standard release channel toolchain names have the following form:

```text
<channel>[-<date>][-<host>]

<channel>       = latest
<date>          = YYYY-MM-DD
<host>          = <target-triple>
```

'channel' is a named release channel. Channel names can be optionally appended
with an archive date, as in `nightly-2014-12-18`, in which case the toolchain
is downloaded from the archive for that date.

Finally, the host may be specified as a target triple.

## Custom toolchains

For most use cases, using the officially packaged toolchains is good enough.

For advanced use cases, `fuelup` allows you to build a toolchain in a
modular manner, and to specify specific versions of components to install.

To init a new, empty toolchain:

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
