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
pub struct Frames {
    channels: usize,
    data: Vec<f32>
}

impl Frames {

    pub(crate) fn new(channels: usize, data: Vec<f32>) -> Self {
        Self { channels, data }
    }

    pub fn new_empty() -> Self {
        Self {
            channels: 0,
            data: vec![],
        }
    }

    pub fn channels(&self) -> usize {
        self.channels
    }

    pub fn num_frames(&self) -> usize {
        self.data.len() / self.channels()
    }
    
    pub fn data(&self) -> &[f32] {
        &self.data
    }

}

