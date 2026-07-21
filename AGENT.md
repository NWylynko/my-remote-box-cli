# AGENT.md

## Building the CLI

This is a Rust CLI whose binary is named `box` (see `[[bin]]` in `Cargo.toml`).

After making changes to the source (`src/main.rs`):

```sh
# Quick check that it compiles
cargo build

# Install the release build to ~/.cargo/bin/box (this is where `box` on PATH lives)
cargo install --path .
```

`cargo build` alone only produces `target/debug/box` and does **not** update the
installed `box` command. To make your changes take effect for the `box` command
on PATH, you must run `cargo install --path .`, which replaces
`~/.cargo/bin/box`.

Verify the install picked up your changes:

```sh
which box   # should print /home/nick/.cargo/bin/box
box list    # or whatever command you changed
```
