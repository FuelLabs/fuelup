# Overrides

`fuelup` automatically determines which [toolchain] to use when one of the installed commands like
`forc` is executed.

Currently, the only way of overriding toolchains is through the [`fuel-toolchain.toml`] file.

# The toolchain file

Using the `fuel-toolchain.toml` file is a way to have projects 'pinned' to specific versions of Sway and have this information reflected in their source repository.

When this file is present, `fuelup` knows to override the default toolchain with the specified toolchain when executing binaries
in the toolchain.

In these cases, the toolchain can be specified in a file called `fuel-toolchain.toml`. These toolchains can either be
the [distributed toolchains or custom toolchains].

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

And the `fuel-toolchain.toml` for an AMM application using a [custom toolchain] might contain:

```toml
[toolchain]
name = "my-awesome-amm-toolchain"
components = ["forc", "fuel-core", "forc-index", "fuel-indexer"]
```

Note that the components key itself and the array value are both optional.

For example, an application using the [`latest`] toolchain might contain:

```toml
[toolchain]
name = "latest"
```

or:

```toml
[toolchain]
name = "latest"
components = []
```

[toolchain]: concepts/toolchains.md
[distributed toolchains or custom toolchains]: concepts/toolchains.md#toolchains
[`fuel-toolchain.toml`]: overrides.md#the-toolchain-file
[custom toolchain]: concepts/toolchains.md#custom-toolchains
[`latest`]: concepts/channels/latest.md

