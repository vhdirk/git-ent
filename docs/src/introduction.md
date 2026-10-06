# Introduction

**git-nest** is a tool to manage projects with (nested) git submodules.

The CLI mirrors familiar git subcommands but operates **recursively**
across every submodule in the tree - so a single `git-nest commit -m "fix"`
creates a commit in every repo that has staged changes, then
automatically updates the submodule pointers in parent repos.

This Rust implementation is built on:

- [`clap`](https://docs.rs/clap) with derive macros for the CLI.
- [`git2`](https://docs.rs/git2) (libgit2) for read-side repository
  introspection.
- [`thiserror`](https://docs.rs/thiserror) for typed, ergonomic errors.
