use super::domain::Frame;

pub trait Player {
    /** 
     * plays frame on playback hardware and blocks until ready 
     * to play next frame. This will likely return before the last
     * sample of [`frame`] is played to allow for gapless playback
     */
    fn play(frame: Frame);
}

pub struct NullPlayer {
}

impl Player for NullPlayer {
    fn play(_frame: Frame) {
        // do nothing
    }
}