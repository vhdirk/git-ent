# Development

## Running tests

```bash
cargo test
```

The integration tests spin up ephemeral git repos (plain, flat
submodules, and nested `main --> mid --> leaf` layouts) inside
`TempDir`s, then invoke the compiled `git-nest` binary via
[`assert_cmd`](https://docs.rs/assert_cmd).

## Building documentation

```bash
# API docs (rustdoc)
cargo doc --no-deps --open

# This mdBook
mdbook serve docs
```

## Source layout

```text
src/
├── cli.rs               - clap CLI definitions
├── commands/            - one module per subcommand
│   ├── add.rs
│   ├── branch.rs
│   ├── clone.rs
│   ├── commit.rs
│   ├── merge.rs
│   ├── push.rs
│   ├── rebase.rs
│   ├── reset.rs
│   ├── restore.rs
│   ├── squash.rs        - NEW: depth-first squash across submodules
│   ├── status.rs
│   └── update.rs
├── error.rs             - GitNestError (thiserror)
├── git.rs               - `git` subprocess helpers
├── lib.rs               - crate root
├── main.rs              - binary entry point
└── repo_tree.rs         - RepoTree & status helpers
```
