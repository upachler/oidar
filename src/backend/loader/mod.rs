use anyhow::Result;

use super::domain::Chunk;

pub trait Loader {

    /** Reads a chunk and blocks until it is available */
    fn read_chunk() -> Result<Chunk>;
}