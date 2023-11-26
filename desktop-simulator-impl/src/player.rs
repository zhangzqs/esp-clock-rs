use std::{thread, time::Duration};

use embedded_tone::{Note, Rest, Player};
use rodio::{dynamic_mixer, source::SineWave, Source};

pub struct RodioPlayer {
    beat_duration: Duration,
    _stream: rodio::OutputStream,
    sink: rodio::Sink,
}

unsafe impl Send for RodioPlayer {}

impl RodioPlayer {
    pub fn new() -> Self {
        let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
        let sink = rodio::Sink::try_new(&stream_handle).unwrap();

        Self {
            beat_duration: Duration::ZERO,
            _stream,
            sink,
        }
    }
}

impl Player for RodioPlayer {

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
