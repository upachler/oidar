use std::sync::mpsc::Receiver;

use anyhow::Result;
use bytes::Bytes;

use crate::domain::backend::ports::{Loader, Decoder, Player};
use crate::domain::backend::models::*;

pub struct DummyLoader;


impl Loader for DummyLoader {

    fn new() -> Self {
        Self {}
    }

    fn set_url(&mut self, url: Option<url::Url>) {
        // do nothing
    }
    fn read_chunk(&self) -> anyhow::Result<Option<Chunk>> {
        let data = [0u8; 32];
        Ok(Some(Chunk::from(Bytes::from_owner(data))))
    }
}


pub struct DummyDecoder {
    chunk: Option<Chunk>,
    input: Receiver<Chunk>,
}

impl DummyDecoder {
}

impl Decoder for DummyDecoder {

    fn new(chunk_input: std::sync::mpsc::Receiver<Chunk>) -> Self {
        Self {
            chunk: None,
            input: chunk_input,
        }
    }

    fn decode(&mut self) -> Result<Frames> {

        if self.chunk.is_none() {
            self.chunk = Some(self.input.recv()?);
        }

        // consume the chunk once all data is read
        self.chunk.take();

        // for each chunk we produce a frame
        Ok(Frames::new_empty())
    }

}

pub struct DummyPlayer{
    input: Receiver<Frames>,
}

impl DummyPlayer {
}

impl Player for DummyPlayer {
    fn new(input: Receiver<Frames>) -> Self {
        Self {
            input
        }
    }
    fn play(&mut self) -> Result<()> {    
        self.input.recv()?;
        Ok(())
    }
}