use std::time::Duration;

use crate::note::{Note, NoteDuration, Rest};

pub trait Player {
    // 每拍的时长
    fn set_beat_duration(&mut self, beat_duration: Duration);

    /// bpm: beat per minute
    fn set_beat_duration_from_bpm(&mut self, bpm: u32, note_duration_as_beat: NoteDuration) {
        // 一分钟有60秒
        // 以给定的音符时值为一拍
        // 一拍的时长为60/bpm秒
        let d: f32 = note_duration_as_beat.into();
        self.set_beat_duration(Duration::from_secs_f32(60.0 / bpm as f32 / d))
    }

    fn play_note(&mut self, note: Note);
    fn play_rest(&mut self, rest: Rest);
    fn play_notes<T>(&mut self, _notes: T)
    where
        T: IntoIterator<Item = Note>,
    {
        unimplemented!("play_notes")
    }
}
