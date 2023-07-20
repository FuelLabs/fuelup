# Process for achieving completion of issue 457

1. In `fuelup-init.sh` use `check_cmd` for `nix`, if it is installed for the user there is no need to run the installation script.
   - find a way to amend their `conf.nix` file to include the fuel binary cache from cachix and the necessary unstable features
2. If `nix` isn't present, run the full `fuel.nix` install script.
3. Possible conflict for the user may be that the binaries produced by the `fuel.nix` flake won't be stored at `/fuelup/bin/`.
   - find out if this will be a problem and if a post install script will be sufficient for moving the installed binaries managed by fuelup
4. Map `fuel.nix` flake commands to the fuelup CLI commands already available.
5. Run tests to ensure all features work as intended.
