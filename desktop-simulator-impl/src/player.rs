use embedded_tone::RawTonePlayer;
use log::debug;
use rodio::{source::SineWave, Source};

pub struct RodioPlayer {
    _stream: rodio::OutputStream,
    sink: rodio::Sink,
}

unsafe impl Send for RodioPlayer {}

// struct MySource {}

// impl Source for MySource {
//     fn current_frame_len(&self) -> Option<usize> {
//         None
//     }

//     fn channels(&self) -> u16 {
//         1
//     }

//     fn sample_rate(&self) -> u32 {
//         44100
//     }

//     fn total_duration(&self) -> Option<std::time::Duration> {
//         None
//     }
// }

impl RodioPlayer {
    pub fn new() -> Self {
        let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
        let sink = rodio::Sink::try_new(&stream_handle).unwrap();

        Self { _stream, sink }
    }
}

impl RawTonePlayer for RodioPlayer {
    fn tone(&mut self, freq: u32) {
        debug!("tone {}", freq);
        // self.sink.append(SineWave::new(freq as f32));
    }

    fn off(&mut self) {}
}
