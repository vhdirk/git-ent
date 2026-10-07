use crate::error::Result;
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct Context {
    // reserved for future context fields
}

impl Context {}

pub trait Cmd {
    fn run(&self, ctx: &Context) -> Result<()>;
}
