use core::time::Duration;

use crate::{
    note::{Note, NoteDuration, Rest},
    SlideNote,
};

pub trait RawTonePlayer {
    fn tone(&mut self, freq: u32);
    fn off(&mut self);
}

pub struct TonePlayer<P, D>
where
    P: RawTonePlayer,
    D: Fn(Duration),
{
    tone_player: P,
    whole_duration: Duration, // 一个全音符的时间
    slide_note_samples: usize,
    delay: D,
}

impl<P, D> TonePlayer<P, D>
where
    P: RawTonePlayer,
    D: Fn(Duration),
{
    pub fn new(player: P, delay: D) -> Self {
        Self {
            tone_player: player,
            whole_duration: Duration::from_secs(4),
            slide_note_samples: 50,
            delay: delay,
        }
    }

    /// bpm: beat per minute
    pub fn set_beat_duration_from_bpm(&mut self, bpm: u32, note_duration_as_beat: NoteDuration) {
        // 一分钟有60秒
        // 以给定的音符时值为一拍
        // 一拍的时长为60/bpm秒
        let d: f32 = note_duration_as_beat.into();
        self.whole_duration = Duration::from_secs_f32(60.0 / bpm as f32 / d);
    }

    pub fn play_note(&mut self, note: Note) {
        let pitch = note.pitch.frequency();
        let duration = self.whole_duration.mul_f32(note.duration.into());
        self.tone_player.tone(pitch);
        (self.delay)(duration);
        self.tone_player.off();
    }
    pub fn play_rest(&mut self, rest: Rest) {
        let dur = self.whole_duration.mul_f32(rest.duration.into());
        (self.delay)(dur);
    }
    pub fn play_slide(&mut self, slide_note: SlideNote) {
        let t = self.whole_duration.mul_f32(slide_note.duration.into());
        let n = (t.as_secs_f32() * self.slide_note_samples as f32) as usize;
        let start_freq = slide_note.start_pitch.frequency() as f32;
        let end_freq = slide_note.end_pitch.frequency() as f32;
        let freq_step = (end_freq - start_freq) / n as f32;

        for i in 0..n {
            let freq = (start_freq + freq_step * i as f32) as u32;
            let t = t / n as u32;
            self.tone_player.tone(freq);
            (self.delay)(t);
            self.tone_player.off();
        }
    }
}
