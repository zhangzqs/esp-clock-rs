use std::{thread, time::Duration};

use embedded_tone::{Note, Player, Rest, SlideNote};
use rodio::{dynamic_mixer, source::SineWave, Source};

pub struct RodioPlayer {
    beat_duration: Duration,
    _stream: rodio::OutputStream,
    sink: rodio::Sink,
    slide_note_samples: usize,
}

unsafe impl Send for RodioPlayer {}

impl RodioPlayer {
    pub fn new() -> Self {
        let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
        let sink = rodio::Sink::try_new(&stream_handle).unwrap();

        Self {
            slide_note_samples: 10,
            beat_duration: Duration::ZERO,
            _stream,
            sink,
        }
    }
}

impl Player for RodioPlayer {
    fn play_slide(&mut self, slide_note: SlideNote) {
        let t = self.beat_duration.mul_f32(slide_note.duration.into());
        let n = (t.as_secs_f32() * self.slide_note_samples as f32) as usize;
        let start_freq = slide_note.start_pitch.frequency();
        let end_freq = slide_note.end_pitch.frequency();
        let freq_step = (end_freq - start_freq) / n as f32;

        for i in 0..n {
            let freq = start_freq + freq_step * i as f32;
            let s = SineWave::new(freq).take_duration(t / n as u32).amplify(1.0);
            self.sink.append(s);
        }
        self.sink.sleep_until_end();
    }

    fn set_beat_duration(&mut self, beat_duration: Duration) {
        self.beat_duration = beat_duration;
    }

    fn play_note(&mut self, note: Note) {
        let t = self.beat_duration.mul_f32(note.duration.into());
        let s = SineWave::new(note.pitch.frequency())
            .take_duration(t)
            .amplify(1.0);
        self.sink.append(s);
        self.sink.sleep_until_end();
    }

    fn play_notes<T>(&mut self, notes: T)
    where
        T: IntoIterator<Item = Note>,
    {
        let (controller, mixer) = dynamic_mixer::mixer::<f32>(1, 48000);
        notes.into_iter().for_each(|note| {
            let t = self.beat_duration.mul_f32(note.duration.into());
            let s = SineWave::new(note.pitch.frequency())
                .take_duration(t)
                .amplify(0.5);
            controller.add(s);
        });
        self.sink.append(mixer);
        self.sink.sleep_until_end();
    }

    fn play_rest(&mut self, rest: Rest) {
        thread::sleep(self.beat_duration.mul_f32(rest.duration.into()));
    }
}
