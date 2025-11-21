# Adding components

Adding components in `fuelup` is often just a small PR to [`components.toml`] in the repo, followed by a
new release of `fuelup`.

## Contributing to `components.toml`

This section describes how you may add support for a binary within `fuelup`.

### Required interface

All `fuelup` components must implement the `--version` argument. `fuelup` uses this to display version information
in its CLI. It should print the name of the executable and the full semver version number. For example:

```sh
$ forc-tx --version
forc-tx 0.44.1
```

### Binary packaged natively in forc

This refers to binaries like `forc-fmt` that are added within the Sway repository.

In this scenario, `fuelup` will already automatically download the tar file from the Sway repository and
unarchive all of them without discrimination. You will be able to use the executables, but certain features
like `fuelup show` may not work until you add them to the [`components.toml`] within the source code. `fuelup`
reads from this TOML file to know which components are supported directly through itself, so this step is
important. You may follow [the PR adding `forc-tx`] as an example of how to add such a component.

### Binary packaged outside of forc

This refers to standalone binaries like `forc-wallet` which are released outside of the Sway `forc-binaries`
bundle (i.e. they are not packaged natively in the Sway repository).

In this case, `fuelup` __will NOT__ download the tar file automatically since it does not know where to
download it from. Currently, we do not support downloading and using arbitrary forc plugins and components, so
information will have to be added to `components.toml` for `fuelup` to know how to handle these. You may follow
[the PR adding `forc-wallet`] as an example of how to add such a component.

Example:

```toml
[component.forc-wallet]
name = "forc-wallet"
tarball_prefix = "forc-wallet"
is_plugin = true
executables = ["forc-wallet"]
repository_name = "forc"
targets = [ "aarch64-unknown-linux-gnu", "x86_64-unknown-linux-gnu", "aarch64-apple-darwin", "x86_64-apple-darwin" ]
publish = true
```

> Note: `forc-wallet` was originally released from its own repository
> [`FuelLabs/forc-wallet`][forc-wallet-repo] up to version `0.15.x`. From
> `0.16.0` onwards it is released from the [`FuelLabs/forc`][forc-repo]
> monorepo as an independent workspace member with its own release cycle.

A description of the configuration keys:

`name`

- Name of the component/plugin (e.g., `"forc-wallet"`, `"fuel-core"`).

`tarball_prefix`

- Fuel binaries conform to the format `<prefix>-<version>-<target>` for tar files. 
  This field specifies the `<prefix>` part (e.g., `"forc-wallet"`, `"forc-binaries"`).

`executables`

- List of executable binaries provided by this component. In most cases, the component 
  name matches the executable, but components like `forc` and `forc-client` may package 
  multiple executables.

`repository_name`

- GitHub repository name where releases are published (e.g., `"sway"`, `"fuel-core"`, `"forc"`).

`legacy_repository_name`

- _Optional_. Legacy repository name for versions before a migration. Used when a component 
  moves between repositories to maintain backward compatibility for older versions.
  For example, `forc-wallet` was originally released from `"forc-wallet"` repository but moved 
  to the `"forc"` monorepo starting with version 0.16.0.

`legacy_before`

- _Optional_. Semver version cutoff (e.g., `"0.16.0"`) before which `legacy_repository_name` 
  is used. Versions below this use the legacy repo; versions at or above use the current repo.
  For `forc-wallet`, versions < 0.16.0 use the legacy `"forc-wallet"` repository, while 
  versions >= 0.16.0 use the `"forc"` monorepo.

`targets`

- List of supported target platforms (e.g., `"linux_amd64"`, `"aarch64-apple-darwin"`).

`is_plugin`

- _Optional_. Boolean indicating if this component is a `forc` plugin (`true`) or 
  standalone binary (`false`).

`publish`

- _Optional_. Boolean declaring if the component is published as a standalone release 
  (`true`) or packaged with `forc` (`false`).

`show_fuels_version`

- _Optional_. Boolean indicating whether to show this component's version in `fuelup show` output.

[`components.toml`]:https://github.com/FuelLabs/fuelup/blob/master/components.toml
[the PR adding `forc-tx`]:https://github.com/FuelLabs/fuelup/pull/363
[the PR adding `forc-wallet`]:https://github.com/FuelLabs/fuelup/pull/195
