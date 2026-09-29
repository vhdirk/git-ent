# Installation

## From source

```bash
cargo install --path .
```

The binary is named `sgit` and is placed in `$CARGO_HOME/bin`
(usually `~/.cargo/bin`).

## Requirements

- Rust 1.80+ (stable).
- A system `git` binary (sgit shells out for write operations).
- A C compiler and `pkg-config` (needed by `libgit2-sys`).

On NixOS / with [direnv](https://direnv.net), the supplied
[`flake.nix`](../../flake.nix) provides all required tooling - just
`cd` into the repo and everything is set up.
