# Documentation update summary

## Why this update was needed

Several pages contradicted Fuelup's current channel implementation. They
described `latest` as the only stable channel, treated nightly as unfinished,
and presented exported toolchain files as exact reproducible locks even when
the generated dated channel may not exist.

## What changed

- Documented all public channels: `latest`, `mainnet`, `testnet`, and
  `nightly`.
- Defined `latest` as the moving mainnet-manifest alias rather than the newest
  upstream Sway release.
- Corrected the distinction between distributed and custom toolchains.
- Explained how project overrides, component versions, and local executable
  paths are resolved.
- Documented that export reads the configured default toolchain rather than a
  project override.
- Clarified that exports record versions but not artifact hashes.
- Added warnings for nonexistent dated `latest`/`nightly` archives, partial
  restoration, non-restorable custom exports, and custom-output overwrites.
- Scoped `fuelup update` to installed undated moving channels; immutable dated
  archives are not updated.
- Added documentation tests that require every runtime public channel to
  appear in the user guides and guard the meaning of `latest`.

This change documents existing implementation behavior. It does not silently
redesign export or restoration semantics.

## Validation

- `cargo fmt --all -- --check` passed.
- The Fuelup mdBook built successfully.
- Documentation tests passed.
- The full locked workspace test suite passed.
- All committed changes pass `git diff --check`.
