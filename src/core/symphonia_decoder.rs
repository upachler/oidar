
use std::io::Read;
use std::io::Seek;
use std::ops::Deref;
use std::slice::Iter;
use std::slice::IterMut;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::sync::Mutex;

use anyhow::Result;
use bytes::Bytes;
use symphonia::core::audio::AudioBufferRef;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::codecs::CODEC_TYPE_NULL;
use symphonia::core::audio::{AudioBuffer, Signal};
use symphonia::core::errors::Error;
use symphonia::core::formats::FormatOptions;
use symphonia::core::formats::FormatReader;
use symphonia::core::io::MediaSource;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::core::units::Duration;

use crate::domain::backend::ports::*;
use crate::domain::backend::models::*;



pub struct SymphoniaDecoder {
    chunk_input: SyncChunkReader,
    inner: Option<Inner>,
}

struct Inner {
    format_reader: Box<dyn FormatReader>,
    decoder: Box<dyn symphonia::core::codecs::Decoder>,
    track_id: u32,
}

struct ChunkReader {
    current_chunk: Chunk,
    current_offset: usize,
    input: Receiver<Chunk>
}


impl ChunkReader {
    pub fn new(input: Receiver<Chunk>) -> Self {
        Self {
            current_chunk: Chunk::from(Bytes::new()),
            current_offset: 0,
            input
        }
    }
}

impl Read for ChunkReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {

        if self.current_offset >= self.current_chunk.data().len() {
            self.current_chunk = match self.input.recv() {
                Ok(chunk) => {
                    self.current_offset = 0;
                    chunk
                },
                Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::BrokenPipe, e)),
            }
        }
        let bytes_to_read = usize::min(buf.len(), self.current_chunk.data().len() - self.current_offset);
        if bytes_to_read > 0 {
            let src = self.current_chunk
            .data()
            .slice(self.current_offset..self.current_offset+bytes_to_read);

            let tgt = &mut buf[0..bytes_to_read];
            tgt.copy_from_slice(&src);
            self.current_offset += bytes_to_read;
        }
        Ok(bytes_to_read)
    }
}

#[derive(Clone)]
struct SyncChunkReader {
    inner: Arc<Mutex<ChunkReader>>,
}

impl SyncChunkReader {
    fn new(input: Receiver<Chunk>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(ChunkReader::new(input)))
        }
    }
}

impl Seek for SyncChunkReader {
    fn rewind(&mut self) -> std::io::Result<()> {
        panic!("rewind not supported")
    }
    fn seek(&mut self, _pos: std::io::SeekFrom) -> std::io::Result<u64> {
        panic!("seek not supported")
    }
    fn stream_position(&mut self) -> std::io::Result<u64> {
        panic!("stream position not supported")
    }
}

impl Read for SyncChunkReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.inner.lock().unwrap().read(buf)
    }
}

impl MediaSource for SyncChunkReader {
    fn byte_len(&self) -> Option<u64> {
        None
    }
    fn is_seekable(&self) -> bool {
        false
    }
}

impl SymphoniaDecoder {
    fn new_inner(&self) -> Inner {
        let reader = self.chunk_input.clone();
        let mss = MediaSourceStream::new(Box::new(reader), Default::default());

        // Create a probe hint using the file's extension. [Optional]
        let mut hint = Hint::new();
        hint.with_extension("mp3");

        // Use the default options for metadata and format readers.
        let meta_opts: MetadataOptions = Default::default();
        let fmt_opts: FormatOptions = Default::default();

        // Probe the media source.
        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &fmt_opts, &meta_opts)
            .expect("unsupported format");

        // Get the instantiated format reader.
        let format = probed.format;

        // Find the first audio track with a known (decodeable) codec.
        let track = format
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            .expect("no supported audio tracks");

        // Use the default options for the decoder.
        let dec_opts: DecoderOptions = Default::default();

        // Create a decoder for the track.
        let decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &dec_opts)
            .expect("unsupported codec");

        // Store the track identifier, it will be used to filter packets.
        let track_id = track.id;
        
        Inner {
            format_reader: format,
            decoder,
            track_id,
        }
    }

    fn into_frames(audio_buffer: AudioBufferRef) -> Result<Frames> {

        let channels = audio_buffer.spec().channels.count();
        let mut buf = SampleBuffer::<f32>::new(5000, audio_buffer.spec().clone());
        buf.copy_interleaved_ref(audio_buffer);

        
        let data = buf.samples()
        .iter()
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
        
        Ok(Frames::new(channels, data))
    }

}

impl Decoder for SymphoniaDecoder{
    fn new(chunk_input: Receiver<Chunk>) -> Self {
        let reader = SyncChunkReader::new(chunk_input);
        
        Self {
            chunk_input: reader,
            inner: None,
        }
    }

    fn decode(&mut self) -> Result<Frames> {
        let inner = match self.inner.as_mut() {
            Some(inner) => inner,
            None => {
                self.inner = Some(self.new_inner());
                self.inner.as_mut().unwrap()
            }
        };

        let format = &mut inner.format_reader;
        let decoder = &mut inner.decoder;
        let track_id = inner.track_id;
        
        // Get the next packet from the media format.
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(Error::ResetRequired) => {
                // The track list has been changed. Re-examine it and create a new set of decoders,
                // then restart the decode loop. 
                self.inner = None;
                return Ok(Frames::new_empty());
            }
            Err(err) => {
                // A unrecoverable error occurred, halt decoding.
                panic!("{}", err);
            }
        };

        // Consume any new metadata that has been read since the last packet.
        while !format.metadata().is_latest() {
            // Pop the old head of the metadata queue.
            format.metadata().pop();

            // Consume the new metadata at the head of the metadata queue.
            if let Some(meta) = format.metadata().current() {
                if !meta.tags().is_empty() {
                    let tags = meta.tags();
                    log::info!("vendor tags found in stream: {tags:?}");
                }
            }
        }

        // If the packet does not belong to the selected track, skip over it.
        if packet.track_id() != track_id {
            return Ok(Frames::new_empty());
        }

        // Decode the packet into audio samples.
        match decoder.decode(&packet) {
            Ok(audio_buffer) => {
                let frame_len = audio_buffer.frames();
                let spec = audio_buffer.spec();
                log::trace!("frame decoded: {frame_len} frames, signal spec: {spec:?}");

                Self::into_frames(audio_buffer)
            }

            Err(Error::IoError(e)) => {
                // The packet failed to decode due to an IO error, skip the packet.
                // return empty frame
                log::warn!("IO error while decoding: {e}");
                return Ok(Frames::new_empty());
            }
            Err(Error::DecodeError(e)) => {
                // The packet failed to decode due to invalid data, skip the packet.
                log::warn!("invalid data encountered while decoding: {e}");
                return Ok(Frames::new_empty());
            }
            Err(e) => {
                // An unrecoverable error occurred, halt decoding.
                panic!("{}", e);
            }
        }
    
    }
}
