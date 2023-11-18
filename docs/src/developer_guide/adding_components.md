# Adding components

Adding components in `fuelup` is often just a small PR to [`components.toml`] in the repo, followed by a
new release of `fuelup`.

## Contributing to components.toml

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

This refers to standalone binaries like `forc-wallet` which have their own repositories.

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
repository_name = "forc-wallet"
targets = [ "aarch64-unknown-linux-gnu", "x86_64-unknown-linux-gnu", "aarch64-apple-darwin", "x86_64-apple-darwin" ]
publish = true
```

A short description of the keys you find above:

`name`

- Name of the component/plugin.

`tarball_prefix`

- Fuel binaries mostly conform to the format `<name>-<version>-<target>` for our tar files. `tarball_prefix` refers to the `<name>` part.

`executables`

- In most cases, the name of the component is the executable itself,
but certain components like `forc` itself and `forc-client` may package multiple executables and therefore have different names.

`repository_name`

- The repo name that contains the releases.

`targets`

- A list of targets the component is released for.

`is_plugin`

- _Optional_. This tells you if this component is a `forc` plugin.

`publish`

- _Optional_. Declares if the component is published as a standalone, or packaged with `forc`.

[`components.toml`]:https://github.com/FuelLabs/fuelup/blob/master/components.toml
[the PR adding `forc-tx`]:https://github.com/FuelLabs/fuelup/pull/363
[the PR adding `forc-wallet`]:https://github.com/FuelLabs/fuelup/pull/195
