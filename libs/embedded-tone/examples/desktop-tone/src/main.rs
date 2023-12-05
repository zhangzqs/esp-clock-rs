use std::{
    thread,
    time::{Duration, Instant},
    vec,
};

use embedded_tone::{
    AbsulateNotePitch, Note, NoteDuration, NoteName, Octave, RawTonePlayer, TonePlayer,
};
use midly::Smf;
use rodio::{source::SineWave, Source};

struct TonePlayerSimulator {
    _stream: rodio::OutputStream,
    sink: rodio::Sink,
}

impl TonePlayerSimulator {
    pub fn new() -> Self {
        let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
        let sink = rodio::Sink::try_new(&stream_handle).unwrap();
        Self { _stream, sink }
    }

    pub fn tone_with_duration(&mut self, freq: u32, delta: u32) {
        self.sink
            .append(SineWave::new(freq as f32).take_duration(Duration::from_millis(delta as u64)));
    }

    pub fn wait(&mut self) {
        self.sink.sleep_until_end();
    }
}

impl RawTonePlayer for TonePlayerSimulator {
    fn tone(&mut self, freq: u32) {
        self.sink.append(SineWave::new(freq as f32));
    }

    fn off(&mut self) {
        println!("off");
        self.sink.stop();
    }
}

const data: &'static [u8] = include_bytes!("../ql.mid");

fn main() {
    let mut player = TonePlayerSimulator::new();
    let smf = Smf::parse(&data).unwrap();
    println!("tracks: {}", smf.tracks.len());
    let t1 = smf.tracks[0].to_vec();
    let mut muse: Vec<(u32, u32)> = vec![];
    for event in t1 {
        if let Some(e) = event.kind.as_live_event() {
            match e {
                midly::live::LiveEvent::Midi { channel, message } => match message {
                    midly::MidiMessage::NoteOff { key, vel } => {
                        println!("off");
                        player.off();
                    }
                    midly::MidiMessage::NoteOn { key, vel } => {
                        println!("key: {}, vel: {}", key, vel);
                        let p = AbsulateNotePitch::from_midi_note_key(key.as_int()).add(-12);
                        let freq = p.frequency();
                        muse.push((freq, 0));
                    }
                    _ => (),
                },
                _ => (),
            }
        }
        let del = event.delta;
        if let Some((_, t)) = muse.last_mut() {
            *t += del.as_int();
        }
    }

    for m in muse {
        player.tone_with_duration(m.0, (m.1 as f32 * 1.5f32) as u32);
    }
    player.wait();
}
