use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use embedded_tone::{AbsulateNotePitch, RawTonePlayer};
use log::{error, info};
use midly::{MetaMessage, Timing, TrackEventKind};

pub fn play_midi<Tone>(
    midi_content: &[u8],
    player: &mut Tone,
    exit_signal: Arc<AtomicBool>,
    mut on_tone_play: impl FnMut(AbsulateNotePitch, u32),
) where
    Tone: RawTonePlayer,
{
    let (header, mut tracks) = midly::parse(midi_content).unwrap();

    // 一个四分音符中包含的tick数
    let tpqn = if let Timing::Metrical(t) = header.timing {
        let tpqn = t.as_int() as u32;
        info!("TPQN: {}", tpqn);
        tpqn
    } else {
        error!("unsupported timing: {:?}", header.timing);
        return;
    };

    // 获取0号轨道
    let track = tracks.next().unwrap().unwrap();

    let mut tempo = 1_000_000;
    let mut current_half_steps = 0;

    let mut current_freq = 0;

    for event in track {
        if exit_signal.load(Ordering::SeqCst) {
            return;
        }
        if event.is_err() {
            continue;
        }

        // 一个四分音符的绝对时间长度，单位为microseconds
        let event = event.unwrap();
        if let TrackEventKind::Meta(e) = event.kind {
            match e {
                MetaMessage::Text(t) => {
                    info!("midi meta text: {}", String::from_utf8_lossy(t));
                }
                MetaMessage::Copyright(t) => {
                    info!("midi meta copyright: {}", String::from_utf8_lossy(t));
                }
                MetaMessage::TrackName(t) => {
                    info!("midi meta track: {}", String::from_utf8_lossy(t));
                }
                MetaMessage::InstrumentName(t) => {
                    info!("midi meta instrument: {}", String::from_utf8_lossy(t));
                }
                MetaMessage::Lyric(t) => {
                    info!("midi meta lyric: {}", String::from_utf8_lossy(t));
                }
                MetaMessage::Tempo(t) => {
                    info!("midi meta tempo: {}", t);
                    tempo = t.as_int();
                }
                MetaMessage::EndOfTrack => {
                    player.off();
                }
                MetaMessage::KeySignature(half_steps, b) => {
                    info!("key signature: half_steps: {}, {}", half_steps, b);
                    current_half_steps = half_steps as i32;
                }
                _ => {
                    info!("no process track event: {:?}", e);
                }
            }
        }

        if let Some(e) = event.kind.as_live_event() {
            // 等待上一事件结束
            let dur =
                Duration::from_micros((event.delta.as_int() as u64 * tempo as u64) / tpqn as u64);
            thread::sleep(dur);

            match e {
                midly::live::LiveEvent::Midi {
                    channel: _,
                    message,
                } => match message {
                    midly::MidiMessage::NoteOff { key: _, vel: _ } => {
                        println!("off");
                        current_freq = 0;
                        player.off();
                    }
                    midly::MidiMessage::NoteOn { key, vel } => {
                        if vel == 0 {
                            current_freq = 0;
                            player.off();
                        } else {
                            println!("key: {}, vel: {}", key, vel);
                            let p = AbsulateNotePitch::from_midi_note_key(key.as_int())
                                .add(current_half_steps);
                            let freq = p.frequency();
                            println!("freq: {}", freq);

                            on_tone_play(p, freq);
                            // 始终倾向于播放更高音调的音，有更高音调的播放更高音调的
                            // 或者如果当前播放的音调已经超过1000ms，那么也可以播放新的音调

                            if freq > current_freq {
                                current_freq = freq;
                                player.tone(freq);
                            }
                        }
                    }
                    _ => (),
                },
                _ => (),
            }
        }
    }
}
