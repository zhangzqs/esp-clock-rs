use std::time::Duration;

use crate::note::{Note, NoteDuration, Rest};

pub trait Player {
    // 每拍的时长
    fn new(beat_duration: Duration) -> Self;

    /// bpm: beat per minute
    fn from_bpm(bpm: u32, note_duration_as_beat: NoteDuration) -> Self
    where
        Self: Sized,
    {
        // 一分钟有60秒
        // 以给定的音符时值为一拍
        // 一拍的时长为60/bpm秒
        let d: f32 = note_duration_as_beat.into();
        Self::new(Duration::from_secs_f32(60.0 / bpm as f32 / d))
    }
    fn play_note(&self, note: Note);
    fn play_rest(&self, rest: Rest);
    fn play_notes<T>(&self, _notes: T)
    where
        T: IntoIterator<Item = Note>,
    {
        unimplemented!("play_notes")
    }
}
