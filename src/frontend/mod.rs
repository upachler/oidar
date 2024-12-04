pub mod easy_repl;

use crate::backend::Backend;

pub trait Frontend {
    fn run(&self, backend: impl Backend);
}