# AGENT.md

## Building the CLI

This is a Rust CLI whose binary is named `box` (see `[[bin]]` in `Cargo.toml`).

After making changes to the source (`src/main.rs`):

```sh
# Quick check that it compiles
cargo build

# Install the release build to ~/.cargo/bin/box (this is where `box` on PATH lives)
cargo install --path .

# Regenerate the zsh completion registration shim (only needed when the set of
# commands/args changes, or after a clap upgrade — project names complete
# dynamically at tab-time and never need regenerating)
COMPLETE=zsh box > ~/.oh-my-zsh/cache/completions/_box
```

`cargo build` alone only produces `target/debug/box` and does **not** update the
installed `box` command. To make your changes take effect for the `box` command
on PATH, you must run `cargo install --path .`, which replaces
`~/.cargo/bin/box`.

The completion shim lives at `~/.oh-my-zsh/cache/completions/_box`, a directory
oh-my-zsh already has in `$fpath`. It uses clap_complete's **dynamic** engine
(`unstable-dynamic` feature): the shim calls `box` back at tab-time with
`COMPLETE=zsh` set, so `box main()` computes candidates live. This is why
`box open <TAB>` completes real project names (the `open` arg has an
`ArgValueCompleter` wired to `util::project_names`). After regenerating the shim,
pick it up in the current shell with: `rm -f ~/.zcompdump* && exec zsh`.

Verify the install picked up your changes:

```sh
which box   # should print /home/nick/.cargo/bin/box
box list    # or whatever command you changed
```
