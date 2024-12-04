use crate::domain::backend::ports::{Loader, Decoder, DecoderState, Player};
use crate::domain::backend::models::*;

pub struct DummyLoader;

impl DummyLoader {
    pub fn new() -> Self{
        Self
    }
}

impl Loader for DummyLoader {
    fn read_chunk() -> anyhow::Result<Chunk> {
        let data = [0u8; 32];
        Ok(Chunk::from(&data as &[u8]))
    }
}


pub struct DummyDecoder {
    chunk: Option<Chunk>,
}

impl DummyDecoder {
    pub fn new() -> Self {
        Self {
            chunk: None
        }
    }
}

impl Decoder for DummyDecoder {
    fn decode(&mut self) -> DecoderState {
        // for each chunk we produce a sample
        match self.chunk {
            Some(_) => {
                self.chunk = None;
                let frame = Frame{};
                DecoderState::FinishedFrame(frame)
            }
            None => {
                DecoderState::NeedChunk
            }
        }
    }

    fn push_chunk(&mut self, chunk: Chunk) {
        self.chunk = Some(chunk)    
    }
}

pub struct DummyPlayer;

impl DummyPlayer {
    pub fn new() -> Self {
        Self
    }
}

impl Player for DummyPlayer {
    fn play(_frame: Frame) {        
    }
}