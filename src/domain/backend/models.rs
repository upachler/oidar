use bytes::Bytes;

/** Chunk of encoded audio data, eg. MP3 */
pub struct Chunk {
    data: Bytes
}

impl std::fmt::Debug for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let size = self.data.len();
        f.write_fmt(format_args!("Chunk {{size={size}}}"))
    }
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


