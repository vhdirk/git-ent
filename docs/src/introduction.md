# Introduction

**sgit** (/ʃɪt/), short for **subgit**, is a tool to manage projects with
(nested) git submodules.

The CLI mirrors familiar git subcommands but operates **recursively**
across every submodule in the tree - so a single `sgit commit -m "fix"`
creates a commit in every repo that has staged changes, then
automatically updates the submodule pointers in parent repos.

This Rust implementation is built on:

- [`clap`](https://docs.rs/clap) with derive macros for the CLI.
- An argument-safe process layer invoking the system `git` CLI for all
  repository operations (discovery, status, hooks, and history operations).
- [`thiserror`](https://docs.rs/thiserror) for typed, ergonomic errors.

By executing the Git CLI directly, sgit leverages Git's native hooks, credential
helpers, line-ending filters, and transport protocols without external C library
dependencies.
