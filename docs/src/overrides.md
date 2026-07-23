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
The `fuel-toolchain.toml` file allows a project to select a distributable base
channel and, optionally, specific component versions or local executables.
Recording these choices improves repeatability, but the file is not a complete
bit-for-bit lockfile: version entries do not include the artifact hashes from a
channel manifest.

When this file is present, `fuelup` overrides the default toolchain when it
executes a managed binary.

The `[toolchain]` channel must be one of:

- `mainnet` or `testnet`; or
- an archived `latest-YYYY-MM-DD` or `nightly-YYYY-MM-DD` channel whose
  manifest was actually published.

Although the install command accepts undated `latest` and `nightly`, those two
names require a date in `fuel-toolchain.toml`. Custom installed toolchain names
cannot be used as the base channel. Locally built tools are supported through
path entries in `[components]`.
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

To override the Forc version while retaining testnet as the base channel:

```text
[toolchain]
channel = "testnet"

[components]
forc = "<FORC_SEMVER>"
```

Replace `<FORC_SEMVER>` with a published semantic version that is compatible
with the network and the other components. Do not infer the current testnet
version from this page; inspect the [testnet manifest].

Alternatively, you can specify local paths to custom binaries. This is useful for development with locally-built tools:

```toml
[toolchain]
channel = "testnet"

[components]
forc = "/usr/local/bin/forc" # absolute path to custom forc binary
fuel-core = "../../../fuel-core/target/release/fuel-core" # relative path from fuel-toolchain.toml location
```

You can also mix version specifications with local paths:

```text
[toolchain]
channel = "testnet"

[components]
forc = "/path/to/custom/forc"
fuel-core = "<FUEL_CORE_SEMVER>"
```

Local paths can be absolute or relative to the `fuel-toolchain.toml` file.
Fuelup validates that a referenced path is a file and is executable.

## Exporting toolchains

<!-- This section should explain how to export toolchains -->
<!-- export:example:start -->
You can generate a `fuel-toolchain.toml` inventory with the `export` command.
Review the generated file before sharing it: export records detected component
versions, but it does not guarantee that the base channel is archived or that
the file can restore the exact same artifacts.
<!-- export:example:end -->

Without a name, export reads the configured default toolchain. It does not
export a project override that happens to be active in the current directory:

```sh
fuelup toolchain export
```

This creates a `fuel-toolchain.toml` file in the current directory containing
the default toolchain's derived channel and detected component versions.

You can also export a specific toolchain by name:

```sh
fuelup toolchain export my-custom-toolchain
```

To export to a custom file path:

```sh
fuelup toolchain export -o my-backup.toml
fuelup toolchain export --output /path/to/my-toolchain.toml
```

### What export preserves

For a `mainnet` or `testnet` toolchain, export preserves the dateless network
channel and records the semantic version of each detected publishable
component. The output has this shape:

```text
[toolchain]
channel = "mainnet"

[components]
forc = "<INSTALLED_FORC_SEMVER>"
fuel-core = "<INSTALLED_FUEL_CORE_SEMVER>"
# Other detected components are included here.
```

The generated component entries do not preserve channel-manifest hashes. Export
also does not preserve the source of locally built binaries; it records a
version when Fuelup can identify one.

### Dated channel caveat

When the installed toolchain is undated `latest` or `nightly`, export currently
turns its name into `latest-<today>` or `nightly-<today>`. Export does not check
whether a matching archive manifest exists.

This is especially important for `latest`: `latest` is an alias for the current
mainnet manifest, but dated `latest` manifests are not published for every
date. A dated nightly is available only when that day's nightly was
successfully published. If the generated archive is missing, Fuelup can still
install individually listed component versions when they are invoked, so a
partial restore may look successful even though the base toolchain was not
restored.

Before committing an export:

1. Verify that its dated channel manifest exists.
2. If the source was undated `latest`, use `mainnet` as the base channel when a
   moving, mainnet-compatible base is appropriate.
3. Test the file with an empty Fuelup home rather than relying on already cached
   components.

Exporting a custom toolchain preserves its custom name in the generated file,
but custom names are not accepted as `[toolchain].channel` values by the
override parser. Such output is an inventory only and is not currently a
restorable project override.

### Overwrite protection

By default, `export` will fail if a `fuel-toolchain.toml` file already exists in the current directory:

```console
$ fuelup toolchain export
error: fuel-toolchain.toml already exists in the current directory. Use --force to overwrite.
```

You can either use the `--force` flag to overwrite the existing file:

```sh
fuelup toolchain export --force
```

Or export to a different path:

```sh
fuelup toolchain export -o backup-toolchain.toml
```

The default-path protection applies only to `fuel-toolchain.toml`. A custom
output path is overwritten if it already exists, even without `--force`.

[toolchain]: concepts/toolchains.md
[`testnet`]: concepts/channels.md#the-testnet-channel
[testnet manifest]: https://github.com/FuelLabs/fuelup/blob/gh-pages/channel-fuel-testnet.toml
