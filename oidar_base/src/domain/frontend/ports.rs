
use crate::domain::backend::ports::Backend;

pub trait Frontend {
    fn run(&self, backend: impl Backend);
}