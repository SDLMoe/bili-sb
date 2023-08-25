# bili-sb

Sponsor Skip for Bilibili

## Build

Requirements:

- Rust Toolchains, always latest stable channel, this project has no MSRV policy. For development, you should install `clippy` and `rustfmt`
  - You can install Rust toolchain with `rustup`, see: [Install Rust - rust-lang.org](https://www.rust-lang.org/tools/install)
  - Run this script to ensure your toolchains are up to date: `rustup update && rustup component add rustfmt clippy`
- Protobuf Compiler, i.e. `protoc`, we need this for bilibili client-side protobuf. See [hyperium/tonic#dependencies](https://github.com/hyperium/tonic/#dependencies) for installation guide.

Once you have all these above installed, you can simply run `cargo build --release` for release artifact.
