//! Git CLI command wrappers.
//!
//! High-level command modules construct typed [`GitCommand`] objects that
//! execute through the local `git` process boundary.

pub mod add;
pub mod branch;
pub mod checkout;
pub mod clone;
pub mod command;
pub mod commit;
pub mod merge;
pub mod push;
pub mod rebase;
pub mod remotes;
pub mod repo;
pub mod reset;
pub mod restore;
pub mod rev_parse;
pub mod status;
pub mod submodule;
pub mod switch;
pub mod symbolic_ref;

pub use add::*;
pub use branch::*;
pub use checkout::*;
pub use clone::*;
pub use command::*;
pub use commit::*;
pub use merge::*;
pub use push::*;
pub use rebase::*;
pub use remotes::*;
pub use repo::*;
pub use reset::*;
pub use restore::*;
pub use rev_parse::*;
pub use status::*;
pub use submodule::*;
pub use switch::*;
pub use symbolic_ref::*;
