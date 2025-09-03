# Overrides

<!-- This section should explain fuelup overrides -->
<!-- overrides:example:start -->
`fuelup` automatically determines which [toolchain] to use when one of the installed commands like
`forc` is executed.

You can override the installed default toolchain using a `fuel-toolchain.toml` file.
<!-- overrides:example:end -->

## The toolchain file

<!-- This section should explain the fuel-toolchain TOML file -->
<!-- toolchain:example:start -->
The `fuel-toolchain.toml` file allows projects to "pin" to a specific set of fuel toolchain component versions.
This greatly improves the reproducibility of a project, as the `fuel-toolchain.toml` contains the set of known,
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

An application using the [`testnet`] toolchain:

```toml
[toolchain]
channel = "testnet"
```

Let's say we have a project on the Fuel testnet network, and we want to try using a different version forc to develop on it:

```toml
[toolchain]
channel = "testnet"

[components]
forc = "0.65.0" # in testnet, forc is pinned to v0.66.1
```

Alternatively, you can specify local paths to custom binaries. This is useful for development with locally-built tools:

```toml
[toolchain]
channel = "testnet"

[components]
forc = "/usr/local/bin/forc" # absolute path to custom forc binary
fuel-core = "../../../fuel-core/target/release/fuel-core" # relative path from fuel-toolchain.toml location
```

You can also mix version specifications with local paths:

```toml
[toolchain]
channel = "testnet"

[components]
forc = "/path/to/custom/forc" # use local custom forc
fuel-core = "0.41.7"         # use specific version of fuel-core
```

Local paths can be either absolute or relative to the `fuel-toolchain.toml` file. When using local paths, `fuelup` will validate that the specified binaries exist and are executable.

## Exporting toolchains

<!-- This section should explain how to export toolchains -->
<!-- export:example:start -->
You can generate a `fuel-toolchain.toml` file from your current toolchain configuration using the `export` command. This is useful for sharing your exact toolchain setup with team members or documenting the specific tool versions used in your project.
<!-- export:example:end -->

To export your currently active toolchain:

```sh
fuelup toolchain export
```

This creates a `fuel-toolchain.toml` file in the current directory containing your active toolchain's channel and installed component versions.

You can also export a specific toolchain by name:

```sh
fuelup toolchain export my-custom-toolchain
```

### Export examples

Exporting a `latest` toolchain produces a file like:

```toml
[toolchain]
channel = "latest-2025-09-03"

[components]
forc = "0.69.1"
fuel-core = "0.45.1"
fuel-core-keygen = "0.45.1"
forc-wallet = "0.15.1"
```

Exporting a custom toolchain preserves the custom name:

```toml
[toolchain]
channel = "my-dev-toolchain"

[components]
forc = "0.69.1"
fuel-core = "0.45.1"
```

### Overwrite protection

By default, `export` will fail if a `fuel-toolchain.toml` file already exists:

```console
$ fuelup toolchain export
error: fuel-toolchain.toml already exists in the current directory. Use --force to overwrite.
```

Use the `--force` flag to overwrite an existing file:

```sh
fuelup toolchain export --force
```

[toolchain]: concepts/toolchains.md
[distributed toolchains]: concepts/toolchains.md#toolchains
[`testnet`]: concepts/channels.md#the-testnet-channel
