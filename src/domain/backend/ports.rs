use anyhow::Result;
use url::Url;
use std::sync::mpsc::Receiver;

use crate::domain::backend::models::Chunk;


pub enum BackendCommand {
    PlayUrl(Url),
    StopPlayback,
    Shutdown,
}

pub enum BackendEvent {
    PlaybackStarted,
    PlaybackStopped,
}

pub trait Backend {
    fn send_command(&self, cmd: BackendCommand);
    fn event_receiver(&self) -> &Receiver<BackendEvent>;
}

pub trait Loader {

    /** Reads a chunk and blocks until it is available */
    fn read_chunk() -> Result<Chunk>;
}

use crate::domain::backend::models::Frame;

pub trait Player {
    /** 
     * plays frame on playback hardware and blocks until ready 
     * to play next frame. This will likely return before the last
     * sample of [`frame`] is played to allow for gapless playback
     */
    fn play(frame: Frame);
}



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