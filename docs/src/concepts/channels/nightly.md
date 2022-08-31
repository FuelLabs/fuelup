# The `nightly` channel

The `nightly` channel is a published TOML file describing successful builds of the `master` branch of `forc` and `fuel-core` for the day. 
These builds are released in the [sway-nightly-binaries] repository and the workflows in that repo start building them every day at **00:00 UTC**.

The `nightly` channel within `fuelup` is updated by a scheduled GitHub workflow that **runs every day at 01:00 UTC**, after builds have finished. 
Note that nightlies might fail to build, in which case it is possible that the `nightly` toolchain may not be available for that day.

You should use `nightly` if you want the latest changes to `master` that have not been officially released yet. 
Keep in mind that compatibility between `forc` and `fuel-core` is not guaranteed here, and you should expect unstable features to break.

[sway-nightly-binaries]: https://github.com/FuelLabs/sway-nightly-binaries/releases
