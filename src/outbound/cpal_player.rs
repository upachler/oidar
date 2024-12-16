use std::sync::mpsc::Receiver;

use cpal::{traits::{DeviceTrait, HostTrait, StreamTrait}, Stream, StreamConfig, SupportedStreamConfig};
use symphonia::core::conv::IntoSample;

use crate::{domain::backend::models::Frames, Player};

pub(crate) struct CpalPlayer {
    config: Box<StreamConfig>,
    stream: Stream,
}

impl Player for CpalPlayer {
    fn new(input: Receiver<Frames>) -> Self {
        let host = cpal::default_host();

        let device = host.default_output_device().expect("no default audio device found, aborting..");
        let config = device.default_output_config()
        .expect("no default outpout config found, aborting..")
        ;
        let sample_format = config.sample_format();
        let sample_rate = config.sample_rate();
        
        let config = Box::new(config.config());
        let data_channels = config.channels as usize;

        let mut frames_offset = 0usize;
        let mut frames: Option<Frames> = None;
        let stream = device.build_output_stream(config.as_ref(), move |data: &mut[f32],context|{
            let mut data_offset = 0;
            
            assert_eq!(0, data.len() % data_channels, "data.len() not a multiple of the number of channels");

            while data_offset < data.len() {
                if frames.is_none() {
                    frames = Some(input.recv().unwrap());
                    frames_offset = 0;
                }

                // shorthand for source Frames struct that we'll pull sample data from
                let f = frames.as_ref().unwrap();

                // calculate sample range we're operating on
                let n = usize::min(f.data().len()-frames_offset, data.len() - data_offset);
                
                // setup source and target ranges that we're operating on
                let source_range = frames_offset .. frames_offset+n;
                let source = &f.data()[source_range];

                let target_range = data_offset..data_offset+n;
                let target = &mut data[target_range];

                // transfer samples to buffer, either
                // * sample by sample, if number of channels are identical in source and target,
                // * or mix all source channels to a mono signal, and write that mono signal to
                //   all target channels
                if data_channels == f.channels() {

                    target.copy_from_slice(source);
/*                  let mut source = source.iter();
                    target.fill_with(||{
                        *source.next().unwrap()
                    });
*/
                } else {
                    let mut source = source.chunks(f.channels());
                    let source_channels = f.channels() as f32;
                    let mut target = target.chunks_mut(data_channels);
                    for _ in 0..n {
                        // mix all channels in source into mono_sample
                        let src_chunk = source.next().unwrap();
                        let src_sum = src_chunk.iter()
                        .map(ToOwned::to_owned)
                        .reduce(|a,b|(a+b))
                        .unwrap();

                        let mono_sample = src_sum / source_channels;

                        // write mono_sample to all target channels
                        target.next()
                        .unwrap()
                        .fill(mono_sample);
                    }
                }
                
                // move data offsets forward and check if we need to drop the
                // current frame because all data was read
                data_offset += n;
                frames_offset += n;
                if frames_offset == f.data().len() - (f.data().len() % f.channels()) {
                    frames = None;
                }
            }
        }, move |err| {
            log::error!("playback error: {err}");
        }, None)
        .unwrap();
        Self {
            config,
            stream,
        }
    }

    fn play(&mut self) -> anyhow::Result<()> {
        self.stream.play()?;
        Ok(())
    }
}