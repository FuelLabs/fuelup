# How fuelup works

<!-- This section should explain the purpose of fuelup -->
<!-- fuelup:example:start -->
`fuelup` is largely inspired by [`rustup`]. It installs and manages multiple Fuel
toolchains and presents them all through a single set of tools within `~/.fuelup/bin`.
<!-- fuelup:example:end -->

Generally, it is not recommended to manually make changes to the fuelup directory, otherwise `fuelup`
might not function as expected. If you have made changes to the directory, we recommend removing
the entire directory and re-installing fuelup with [fuelup-init].

## Proxies

On the surface, the installed executables seem to live in `~/.fuelup/bin`, but the [`forc`], [`fuel-core`]
and other executables installed in `~/.fuelup/bin` are actually not the real binaries
but are all just symlinks to `fuelup` itself! `fuelup` contains logic to act as a
[proxy] for the real binaries, so that it can change its behavior based on what component
is being called. This is how `fuelup` can switch between toolchains.

A common mistake is to directly move binaries into the `bin` directory, which would
break the behavior of `fuelup`.

## Store

All actual executables are installed within the _[store]_. This is usually `~/.fuelup/store`.
This is where the real binaries are installed and cached to be used in _[toolchains]_
and _[overrides]_ through symlinks. `fuelup` will always check the store for existing components
before trying to install them - which means you can avoid the download entirely if something
is already cached within the store!

### Example

<!-- This section should give an example of how fuelup works -->
<!-- fuelup_example:example:start -->
To give an example of how this all works together: imagine typing `forc build` in your terminal.
This call invokes `forc` (which is actually `fuelup`) within the fuelup bin directory, which in
turn executes the correct version of `forc` based on either an override file (`fuel-toolchain.toml`)
or your currently active toolchain (in order of priority).
<!-- fuelup_example:example:end -->

[fuelup-init]: ../installation/index.html#quickstart
[`rustup`]: https://github.com/rust-lang/rustup
[`forc`]: https://fuellabs.github.io/sway/master/book/forc/index.html
[`fuel-core`]: https://github.com/FuelLabs/fuel-core
[proxy]: proxies.md
[store]: store.md
[toolchains]: toolchains.md
[overrides]: ../overrides.md
