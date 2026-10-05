//! sgit library: the recursive-submodule-git tool.
//!
//! The public surface consists of:
//! - [`RepoTree`] - the core abstraction modelling a top-level repo and
//!   every (nested) submodule as a depth-first traversable tree.
//! - [`cli`] - the `clap`-powered CLI layer that wires subcommands to
//!   functions in [`commands`].
//! - [`commands`] - one module per subcommand (`clone`, `status`, `add`,
//!   `commit`, `push`, `squash`, …).
//! - [`SgitError`] - the crate-wide error type.

pub mod cli;
pub mod commands;
pub mod config;
pub mod error;
pub mod git;
pub mod repo_tree;
pub mod repo;

pub use error::{Result, SgitError};
pub use repo_tree::RepoTree;
