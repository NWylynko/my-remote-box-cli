#!/usr/bin/env bash
#
# Build, install, and set up shell completions for the `box` CLI.
#
#   ./install.sh
#
# Steps:
#   1. cargo install --path .   -> release build into ~/.cargo/bin/box
#   2. write the zsh completion shim into oh-my-zsh's fpath
#   3. clear the completion dump so the shim is picked up on next shell start
#
set -euo pipefail

# Run from the repo root regardless of where the script is invoked.
cd "$(dirname "$0")"

echo "==> Building and installing box"
cargo install --path .

# oh-my-zsh already has $ZSH/cache/completions in its $fpath.
ZSH_DIR="${ZSH:-$HOME/.oh-my-zsh}"
COMPLETIONS_DIR="${ZSH_CACHE_DIR:-$ZSH_DIR/cache}/completions"

if [ -d "$ZSH_DIR" ]; then
    echo "==> Installing zsh completions -> $COMPLETIONS_DIR/_box"
    mkdir -p "$COMPLETIONS_DIR"
    # Dynamic completion shim: it calls `box` back at tab-time, so project
    # names (e.g. for `box open`) stay live without regenerating this file.
    COMPLETE=zsh box > "$COMPLETIONS_DIR/_box"

    # Completions are cached in the compdump; drop it so compinit rebuilds
    # and sees the (possibly changed) command set on the next shell start.
    rm -f "$HOME"/.zcompdump* 2>/dev/null || true
    echo "==> Done. Run 'exec zsh' to load completions in this shell."
else
    echo "==> oh-my-zsh not found at $ZSH_DIR — skipping completions."
    echo "    For zsh completions, run: COMPLETE=zsh box > /path/in/your/fpath/_box"
    echo "==> Done."
fi
