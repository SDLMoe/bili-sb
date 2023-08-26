# bili-sb

Sponsor Skip for Bilibili

## Build

Requirements:

- Rust Toolchains, always latest stable channel, this project has no MSRV policy. For development, you should install `clippy` and `rustfmt`
  - You can install Rust toolchain with `rustup`, see: [Install Rust - rust-lang.org](https://www.rust-lang.org/tools/install)
  - Run this script to ensure your toolchains are up to date: `rustup update && rustup component add rustfmt clippy`
- Protobuf Compiler, i.e. `protoc`, we need this for bilibili client-side protobuf. See [hyperium/tonic#dependencies](https://github.com/hyperium/tonic/#dependencies) for installation guide.

Once you have all these above installed, for building release artifact, you can simply run:

```bash
cargo build --release --package bili-sb
```

## Contribution

You should install Git hooks for local inspection:

```bash
/bin/rm -rf .git/hooks && ln -s ../.git-hooks ./.git/hooks
```

Your commits should pass the CI including `cargo fmt` and `cargo clippy` check.

We use [Conventional Commit](https://www.conventionalcommits.org/en/v1.0.0/) for commit message, and the first line length of commit message should be less than or equal to 72.
