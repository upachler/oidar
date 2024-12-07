use bytes::Bytes;

/** Chunk of encoded audio data, eg. MP3 */
#[derive(Debug)]
pub struct Chunk {
    data: Bytes
}

impl Chunk {
    pub fn data(&self) -> &Bytes {
        &self.data
    }

}

impl From<Bytes> for Chunk {
    fn from(value: Bytes) -> Self {        
        Self {data: value}
    }
}

/** a frame of PCM coded audio data, ready for playback on PCM audio device */
#[derive(Debug)]
pub struct Frame {
}


