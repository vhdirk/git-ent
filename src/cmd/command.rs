use crate::error::Result;
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct Context {
    pub workdir: Option<PathBuf>,
}

impl Context {}

pub trait Cmd {
    fn run(&self, ctx: &Context) -> Result<()>;
}
