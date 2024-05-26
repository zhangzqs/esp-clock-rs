use std::{
    cell::RefCell,
    rc::Rc,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use embedded_tone::AbsulateNotePitch;
use log::{error, info};
use midly::{MetaMessage, Timing, TrackEventKind};

use crate::proto::*;

use self::ipc::{BuzzerClient, MidiPlayerClient};

fn living_play_midi(
    midi_content: &[u8],
    exit_signal: Arc<AtomicBool>,
    mut on_tone_play: impl FnMut(Option<AbsulateNotePitch>),
    mut delay: impl FnMut(Duration),
) {
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

    let mut current_tone = None;

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
                    on_tone_play(None);
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
            delay(dur);

            match e {
                midly::live::LiveEvent::Midi {
                    channel: _,
                    message,
                } => match message {
                    midly::MidiMessage::NoteOff { key: _, vel: _ } => {
                        current_tone = None;
                        on_tone_play(current_tone);
                    }
                    midly::MidiMessage::NoteOn { key, vel } => {
                        if vel == 0 {
                            current_tone = None;
                            on_tone_play(current_tone);
                        } else {
                            let p = AbsulateNotePitch::from_midi_note_key(key.as_int())
                                .add(current_half_steps);
                            // 始终倾向于播放更高音调的音，有更高音调的播放更高音调的
                            if p.to_midi_note_key()
                                > current_tone.map(|x| x.to_midi_note_key()).unwrap_or(0)
                            {
                                current_tone = Some(p);
                                on_tone_play(current_tone);
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

/// 将midi二进制解析转换为[频率, 延续时间]序列
fn midi_to_freq_and_dur_series(midi_content: &[u8]) -> ToneSeries {
    let src_ret = Rc::new(RefCell::new(vec![]));
    let src_last_dur = Rc::new(RefCell::new(Duration::ZERO));
    let src_last_freq = Rc::new(RefCell::new(0));
    living_play_midi(
        midi_content,
        Arc::new(AtomicBool::new(false)),
        {
            let ret = src_ret.clone();
            let last_dur = src_last_dur.clone();
            let last_freq = src_last_freq.clone();
            move |t| {
                let mut ret = ret.borrow_mut();
                ret.push((*last_freq.borrow(), ToneDuration::from(*last_dur.borrow())));
                *last_dur.borrow_mut() = Duration::ZERO;
                *last_freq.borrow_mut() = t.map(|x| x.frequency() as u16).unwrap_or(0);
            }
        },
        {
            let last_dur = src_last_dur.clone();
            move |dur| {
                *last_dur.borrow_mut() += dur;
            }
        },
    );
    let src_ret_ref = src_ret.borrow();
    ToneSeries(src_ret_ref.clone())
}

// static mid: &[u8] = include_bytes!("../../../music/If_I_Can_Stop_One_Heart_From_Breaking.mid");
static mid: &[u8] = include_bytes!("../../../a.mid");

pub struct MidiPlayerService {}

impl MidiPlayerService {
    pub fn new() -> Self {
        Self {}
    }
}

impl Node for MidiPlayerService {
    fn node_name(&self) -> NodeName {
        NodeName::Midi
    }

    fn handle_message(&self, ctx: Rc<dyn Context>, msg: MessageWithHeader) -> HandleResult {
        let seq = msg.seq;
        match msg.body {
            Message::Lifecycle(LifecycleMessage::Init) => {
                MidiPlayerClient(ctx.clone()).play(
                    mid.to_vec(),
                    Box::new(|r| {
                        info!("midi播放完毕: {:?}", r);
                    }),
                );
                slint::Timer::single_shot(Duration::from_secs(10), move || {
                    MidiPlayerClient(ctx).off();
                });
            }
            Message::Midi(msg) => {
                let cli = BuzzerClient(ctx.clone());
                match msg {
                    MidiMessage::PlayRequest(bs) => {
                        cli.tone_series(
                            midi_to_freq_and_dur_series(&bs.0),
                            Box::new(move |is_finished| {
                                ctx.async_ready(
                                    seq,
                                    Message::Midi(MidiMessage::PlayResponse(is_finished)),
                                );
                            }),
                        );
                        return HandleResult::Pending;
                    }
                    MidiMessage::Off => {
                        cli.off();
                        return HandleResult::Finish(Message::Empty);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        HandleResult::Discard
    }
}
