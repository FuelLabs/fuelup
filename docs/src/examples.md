# Examples

| Command                                   | Description                                                        |
| ----------------------------------------- | ------------------------------------------------------------------ |
| `fuelup toolchain install latest`         | Installs the toolchain distributed by the `latest` channel         |
| `fuelup toolchain new my_toolchain`       | Creates a new toolchain named 'my_toolchain'                       |
| `fuelup toolchain uninstall my_toolchain` | Uninstalls the toolchain named 'my_toolchain'                      |
| `fuelup default my_toolchain`             | Set 'my_toolchain' as the active toolchain                         |
| `fuelup component add forc`               | Adds _[forc]_ to the currently active custom toolchain             |
| `fuelup component add fuel-core@0.9.5`    | Adds _[fuel-core]_ v0.9.5 to the currently active custom toolchain |
| `fuelup component remove forc`            | Removes _forc_ from the currently active custom toolchain          |
| `fuelup self update`                      | Updates _fuelup_                                                   |
| `fuelup check`                            | Checks for updates to official toolchains                          |
| `fuelup toolchain help`                   | Show the `help` page for a subcommand (like `toolchain`)           |

[forc]: https://github.com/FuelLabs/sway/tree/master/forc
[fuel-core]: https://github.com/FuelLabs/fuel-core
