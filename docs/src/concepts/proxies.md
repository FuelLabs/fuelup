# Proxies

`fuelup` provides wrappers for the common Sway toolchain
[components]. These are called _proxies_ and represent commands
which are provided by the components themselves.

This is how `fuelup` knows to differentiate between different
toolchains and different versions of the same components, since
running a component's command will use `fuelup` as a proxy to execute.

This allows the developer to switch smoothly between different
toolchains if they are working on different projects.

[components]: components.md
