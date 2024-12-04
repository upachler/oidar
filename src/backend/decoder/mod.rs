
use super::domain::{Chunk,Frame};

pub enum DecoderState {
    /** 
     * Stream is decoded until end of chunk, provide new chunk before calling
     * [Decoder::decode()] again.
     */
    NeedChunk,
    /**
     * Decoded current chunk until the end of the frame was reached. Next
     * call to [Decoder::decode()] will start a new frame.
     */
    FinishedFrame(Frame),
}

pub trait Decoder {
    fn decode(&mut self) -> DecoderState;
    fn push_chunk(&mut self, chunk: Chunk);
}