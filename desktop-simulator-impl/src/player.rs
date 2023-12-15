use std::{f64::consts::PI, fs::File, io::Write, time::Duration};

use embedded_tone::RawTonePlayer;
use log::debug;
use rodio::{source::SineWave, OutputStream, Source};

pub struct RodioPlayer {
    stream: rodio::OutputStream,
    stream_handle: rodio::OutputStreamHandle,
    sink: rodio::Sink,
}

impl RodioPlayer {
    pub fn new() -> Self {
        let (stream, stream_handle) = rodio::OutputStream::try_default().unwrap();

        let sink = rodio::Sink::try_new(&stream_handle).unwrap();

        Self {
            sink,
            stream,
            stream_handle,
        }
    }
}

impl RawTonePlayer for RodioPlayer {
    fn tone(&mut self, freq: u32) {
        debug!("tone {}", freq);
        self.sink.stop();
        let now = std::time::Instant::now();
        self.sink.append(SineWave::new(freq as f32));
        debug!("append takes {:?}", now.elapsed());
    }

    fn off(&mut self) {
        self.sink.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_tone() {
        let mut player = RodioPlayer::new();
        let src = "0 830\n790 0\n0 659\n3309 0\n1 440\n102 0\n104 391\n68 0\n1 440\n68 0\n1 391\n68 0\n1 329\n102 0\n104 391\n103 0\n104 440\n102 0\n104 391\n68 0\n1 440\n68 0\n1 391\n68 0\n1 329\n102 0\n104 391\n102 0\n104 440\n102 0\n104 391\n68 0\n1 440\n68 0\n1 391\n68 0\n1 329\n103 0\n104 391\n102 0\n104 440\n309 0\n311 391\n102 0\n104 440\n103 0\n104 391\n68 0\n1 440\n68 0\n1 391\n68 0\n1 329\n102 0\n104 391\n102 0\n105 440\n102 0\n105 391\n68 0\n1 440\n68 0\n1 391\n68 0\n1 329\n102 0\n104 391\n102 0\n104 440\n103 0\n104 391\n68 0\n1 440\n68 0\n1 391\n68 0\n1 329\n102 0\n104 391\n102 0\n104 440\n309 0\n311 391\n102 0\n104 880\n102 0\n105 880\n2688 0\n208 1046\n102 0\n104 1174\n1654 0\n208 1318\n111 0\n104 1318\n112 0\n104 1318\n112 0\n104 1318\n215 0\n208 1318";
        src.split("\n")
            .map(|x| {
                let mut x = x.split_whitespace();
                let d = x.next().unwrap().parse::<u64>().unwrap();
                let f = x.next().unwrap().parse::<u32>().unwrap();
                (d, f)
            })
            .for_each(|(d, f)| {
                println!("{} {}", d, f);
                if d > 10 {
                    std::thread::sleep(Duration::from_millis(d));
                }
                player.tone(f);
            });
    }
}
