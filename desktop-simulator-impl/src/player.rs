use std::{thread, time::Duration};

use embedded_tone::{Note, RawTonePlayer, Rest, SlideNote};
use rodio::{dynamic_mixer, source::SineWave, Source};

pub struct RodioPlayer {
    _stream: rodio::OutputStream,
    sink: rodio::Sink,
}

unsafe impl Send for RodioPlayer {}

impl RodioPlayer {
    pub fn new() -> Self {
        let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
        let sink = rodio::Sink::try_new(&stream_handle).unwrap();

        Self { _stream, sink }
    }
}

impl RawTonePlayer for RodioPlayer {
    fn tone(&mut self, freq: u32) {
        self.off();
        self.sink.append(SineWave::new(freq as f32));
    }

    fn off(&mut self) {
        self.sink.stop();
    }
}
