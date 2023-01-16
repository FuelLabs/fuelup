# Overrides

`fuelup` automatically determines which [toolchain] to use when one of the installed commands like
`forc` is executed.

You can override the installed default toolchain using a `fuel-toolchain.toml` file.

## The toolchain file

The fuel-toolchain.toml file allows projects to "pin" to a specific set of fuel toolchain component versions.
This greatly improves the reproducibility of a project, as the fuel-toolchain.toml contains the set of known,
working versions for each tool used to build it.

When this file is present, `fuelup` will override the default toolchain with the specified toolchain when executing binaries
in the toolchain.

In these cases, the toolchain can be specified in a file called `fuel-toolchain.toml`. These toolchains can only be
the [distributed toolchains] at this point in time.

Here's what a sample project might look like:

```console
$ tree -L 1 # 'tree -L 1' shows the project structure up to the depth of 1
.
├── Cargo.toml
├── Forc.toml
├── fuel-toolchain.toml
├── project
├── README.md
└── SPECIFICATION.md
```

### Examples

An application using the [`beta-2`] toolchain:

```toml
[toolchain]
channel = "beta-2"
```

Let's say we have a project on the `beta-2` testnet, and we want to try using the
latest (at time of writing) forc to develop on it:

```toml
[toolchain]
channel = "beta-2"

[components]
forc = "0.33.0" # in beta-2, forc is pinned to v0.31.1
```

[toolchain]: concepts/toolchains.md
[distributed toolchains]: concepts/toolchains.md#toolchains
[`beta-2`]: concepts/channels/beta-2.md
