use anyhow::Result;
use url::Url;
use std::sync::mpsc::Receiver;

use crate::domain::backend::models::Chunk;

#[derive(Debug)]
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
    fn send_command(&self, cmd: BackendCommand) -> Result<()>;
    fn event_receiver(&self) -> &Receiver<BackendEvent>;
}

pub trait Loader : Send {
    fn new() -> Self;

    // set the url where th load the stream from. If changed,
    // the next call to [read_chunk()] will load from that
    // new url, starting from the beginning.
    fn set_url(&mut self, url: Option<Url>);

    /** Reads a chunk and blocks until it is available */
    fn read_chunk(&self) -> Result<Option<Chunk>>;
}

use crate::domain::backend::models::Frames;

pub trait Player {
    fn new(input: Receiver<Frames>) -> Self; 

    /** 
     * reads single frame, send to playback hardware and block until ready 
     * to play next frame. This will likely return before the last
     * sample of [`frame`] is played to allow for gapless playback
     */
    fn play(&mut self) -> Result<()>;
}


pub trait Decoder {
    fn new(chunk_input: Receiver<Chunk>) -> Self;
    fn decode(&mut self) -> Result<Frames>;
}