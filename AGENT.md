# AGENT.md

## Building the CLI

This is a Rust CLI whose binary is named `box` (see `[[bin]]` in `Cargo.toml`).

After making changes to the source, run the install script — it rebuilds,
reinstalls, and refreshes shell completions in one shot:

```sh
./install.sh
```

Then `exec zsh` to load completions in the current shell.

The script runs `cargo install --path .` (release build into `~/.cargo/bin/box`,
where `box` on PATH lives), writes the zsh completion shim, and clears the
compdump. Doing these by hand instead:

```sh
cargo build                                            # just a compile check -> target/debug/box
cargo install --path .                                 # updates the installed box
COMPLETE=zsh box > ~/.oh-my-zsh/cache/completions/_box # regen completion shim
```

`cargo build` alone only produces `target/debug/box` and does **not** update the
installed `box` command — that requires `cargo install --path .`, which replaces
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
