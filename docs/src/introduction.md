# Introduction

**sgit** (/ʃɪt/), short for **subgit**, is a tool to manage projects with
(nested) git submodules.

The CLI mirrors familiar git subcommands but operates **recursively**
across every submodule in the tree - so a single `sgit commit -m "fix"`
creates a commit in every repo that has staged changes, then
automatically updates the submodule pointers in parent repos.

This Rust implementation is built on:

- [`clap`](https://docs.rs/clap) with derive macros for the CLI.
- [`git2`](https://docs.rs/git2) (libgit2) for read-side repository
  introspection.
- [`thiserror`](https://docs.rs/thiserror) for typed, ergonomic errors.
- A thin shell-out layer for the few operations (merge, rebase, push,
  restore, add, …) where the git CLI's semantics are simpler and more
  robust than re-implementing them on top of libgit2.

> Why both? libgit2 is excellent for *reading* a repository (status,
> branches, submodules, merge-base computations); the git CLI handles
> *all* the corner cases of write-side operations (hooks, credential
> helpers, line-endings, renames) so sgit doesn't have to.
