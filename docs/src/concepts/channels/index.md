# Channels

`fuelup` adopts a simplified version of `rustup` [channels](https://rust-lang.github.io/rustup/concepts/channels.html). Currently, only the `latest` channel will be published and serve as a source of distribution of Fuel toolchain binaries.

| Channel      | Source          | Integration Tested   | Update Frequency         | Available |
| ------------ | --------------- | -------------------- | ------------------------ | --------- |
| **[latest]** | published bins  | ✔️                   | checked every 30 minutes | ✔️        |
| **nightly**  | `master` branch | ✔️                   | nightly                  | ➖        |
| **beta**     | published bins  | ✔️                   | every 6 weeks            | ➖        |
| **stable**   | published bins  | ✔️ + successful beta | every 6 weeks            | ➖        |

[latest]: latest.html
