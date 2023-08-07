# Overrides

<!-- This section should explain fuelup overrides -->
<!-- overrides:example:start -->
`fuelup` automatically determines which [toolchain] to use when one of the installed commands like
`forc` is executed.

You can override the installed default toolchain using a `fuel-toolchain.toml` file.
<!-- overrides:example:end -->

## The toolchain file

<!-- This section should explain the fuel-toolchain.toml file -->
<!-- toolchain:example:start -->
The fuel-toolchain.toml file allows projects to "pin" to a specific set of fuel toolchain component versions.
This greatly improves the reproducibility of a project, as the fuel-toolchain.toml contains the set of known,
working versions for each tool used to build it.

When this file is present, `fuelup` will override the default toolchain with the specified toolchain when executing binaries
in the toolchain.

In these cases, the toolchain can be specified in a file called `fuel-toolchain.toml`. These toolchains can only be
the [distributed toolchains] at this point in time.
<!-- toolchain:example:end -->

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

An application using the [`beta-3`] toolchain:

```toml
[toolchain]
channel = "beta-3"
```

Let's say we have a project on the `beta-3` testnet, and we want to try using a different version forc to develop on it:

```toml
[toolchain]
channel = "beta-3"

[components]
forc = "0.36.0" # in beta-3, forc is pinned to v0.37.3
```

[toolchain]: concepts/toolchains.md
[distributed toolchains]: concepts/toolchains.md#toolchains
[`beta-3`]: concepts/channels.md#the-beta-3-channel
