use crate::error::Result;

#[derive(Debug, Clone, Default)]
pub struct Context {
    // reserved for future context fields
}

impl Context {}

pub trait Cmd {
    fn run(&self, ctx: &Context) -> Result<()>;
}
