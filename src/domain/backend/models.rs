/** Chunk of encoded audio data, eg. MP3 */
pub struct Chunk {
    data: Vec<u8>
}

impl Chunk {
    pub fn data(&self) -> &[u8] {
        todo!();
    }

}

impl From<&[u8]> for Chunk {
    fn from(value: &[u8]) -> Self {        
        let data = Vec::from(value.clone());
        Self {data}
    }
}

/** a frame of PCM coded audio data, ready for playback on PCM audio device */
pub struct Frame {
}


