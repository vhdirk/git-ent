//! Subcommand implementations.
//!
//! Each submodule exposes a `run(...)` function that performs one CLI
//! subcommand end-to-end: discover the [`crate::RepoTree`], walk it
//! depth-first, perform the action and print progress to stdout/stderr.

pub mod add;
pub mod branch;
pub mod checkout;
pub mod clone;
pub mod commit;
// pub mod merge;
pub mod push;
// pub mod rebase;
// pub mod reset;
// pub mod restore;
// pub mod squash;
pub mod status;
// pub mod switch;
// pub mod update;
