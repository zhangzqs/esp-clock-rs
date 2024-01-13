use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::channel,
        Arc, Mutex,
    },
    thread,
    time::{Duration},
};

use crate::{interface::LEDController, AppWindow, MusicItemInfo};
use embedded_tone::{AbsulateNotePitch, RawTonePlayer};
use include_dir::{include_dir, Dir};
use log::{error, info};
use midly::{MetaMessage, Timing, TrackEventKind};
use slint::Weak;
use std::sync::mpsc;

static MUSIC_DIST: Dir = include_dir!("music-dist");

enum MusicAppEvent {
    Exit,
    Switch {
        info: MusicItemInfo,
        data: &'static [u8],
    },
}

pub struct MusicApp<TONE, LC>
where
    TONE: RawTonePlayer + 'static + Send,
    LC: LEDController + 'static + Send,
{
    app: Weak<AppWindow>,
    tone_player: Arc<Mutex<TONE>>,
    led: Arc<Mutex<LC>>,

    event_sender: mpsc::Sender<MusicAppEvent>,
    event_receiver: Arc<Mutex<mpsc::Receiver<MusicAppEvent>>>,

    join_handle: Option<thread::JoinHandle<()>>,
}

impl<TONE, LC> MusicApp<TONE, LC>
where
    TONE: RawTonePlayer + 'static + Send,
    LC: LEDController + 'static + Send,
{
    pub fn new(app: Weak<AppWindow>, tone_player: Arc<Mutex<TONE>>, led: Arc<Mutex<LC>>) -> Self {
        let (tx, rx) = channel();
        Self {
            app,
            tone_player,
            led,
            join_handle: None,
            event_sender: tx,
            event_receiver: Arc::new(Mutex::new(rx)),
        }
    }

    fn play_midi(
        midi_content: &[u8],
        _app: Weak<AppWindow>,
        tone_player: Arc<Mutex<TONE>>,
        led: Arc<Mutex<LC>>,
        exit_signal: Arc<AtomicBool>,
    ) {
        let mut player = tone_player.lock().unwrap();
        let mut led = led.lock().unwrap();

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
        let mut max_freq = 0;
        let mut min_freq = 10000;

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
                let dur = Duration::from_micros(
                    (event.delta.as_int() as u64 * tempo as u64) / tpqn as u64,
                );
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
                                max_freq = max_freq.max(freq);
                                min_freq = min_freq.min(freq);
                                let s = (freq - min_freq) as f32 / (max_freq - min_freq) as f32;
                                println!("screen: {}", s);
                                led.set_brightness_percent(s);
                                // app.upgrade_in_event_loop(move |ui| {
                                //     ui.set_music_page_note(format!("{:?}", p).into());
                                //     ui.set_music_page_percent(s);
                                // })
                                // .unwrap();

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

    pub fn enter(&mut self) {
        let tone_player = self.tone_player.clone();
        let led = self.led.clone();
        let app = self.app.clone();
        let event_recv = self.event_receiver.clone();

        self.join_handle = Some(thread::spawn(move || {
            let app = app.clone();
            let tone_player = tone_player.clone();
            let led = led.clone();

            let event_recv = event_recv.lock().unwrap();
            let mut current_play_thread: Option<thread::JoinHandle<()>> = None;
            let exit_signal = Arc::new(AtomicBool::new(false));

            for app_event in event_recv.iter() {
                let exit_signal = exit_signal.clone();
                let app = app.clone();
                let tone_player = tone_player.clone();
                let led = led.clone();

                match app_event {
                    MusicAppEvent::Exit => {
                        if let Some(j) = current_play_thread {
                            // 存在播放线程，发送退出信号，等待退出
                            exit_signal.store(true, Ordering::SeqCst);
                            j.join().unwrap();
                        }
                        return;
                    }
                    MusicAppEvent::Switch { info, data } => {
                        exit_signal.store(true, Ordering::SeqCst);
                        if let Some(current_play_thread) = current_play_thread {
                            // 存在播放线程，发送退出信号，等待退出
                            current_play_thread.join().unwrap();
                        }
                        exit_signal.store(false, Ordering::SeqCst);
                        let app = app.clone();
                        // 开启新的播放线程，播放新的音乐
                        current_play_thread = Some(thread::spawn(move || {
                            Self::play_midi(data, app.clone(), tone_player, led, exit_signal.clone());
                            if !exit_signal.load(Ordering::SeqCst) {
                                // 不存在退出信号，自然播放完毕
                                app.upgrade_in_event_loop(move |ui| {
                                    // 将会自动播放下一曲
                                    ui.invoke_music_page_on_play_done(info);
                                });
                            }
                        }));
                    }
                }
            }
        }));
    }

    pub fn exit(&mut self) {
        info!("recv exit signal");
        self.event_sender.send(MusicAppEvent::Exit).unwrap();
        if let Some(join_handle) = self.join_handle.take() {
            join_handle.join().unwrap();
        }
        // 退出前关闭音调
        self.tone_player.lock().unwrap().off();
        info!("music exit");
    }

    pub fn play(&mut self, item: MusicItemInfo) -> bool {
        if let Some(f) = MUSIC_DIST.get_file(item.path.as_str()) {
            self.event_sender
                .send(MusicAppEvent::Switch {
                    info: item,
                    data: f.contents(),
                })
                .unwrap();
            true
        } else {
            false
        }
    }

    fn play_123() {
        // let player_ref = player.clone();
        // thread::spawn(move || {
        //     let mut player = player_ref.lock().unwrap();
        //     use embedded_tone::{
        //         Guitar,
        //         GuitarString::*,
        //         NoteDuration::{Eighth, Half, HalfDotted, Quarter, Sixteenth},
        //         NoteName::*,
        //         Octave::*,
        //         Rest,
        //     };

        //     let mut guitar = Guitar::default();

        //     for i in (4..12).step_by(2) {
        //         guitar.set_capo_fret(i);
        //         player.set_beat_duration_from_bpm(120, Quarter);

        //         player.play_slide(SlideNote {
        //             start_pitch: guitar.to_absulate_note_pitch(S3, 2),
        //             end_pitch: guitar.to_absulate_note_pitch(S3, 8),
        //             duration: Quarter,
        //         });
        //         player.play_slide(SlideNote {
        //             start_pitch: guitar.to_absulate_note_pitch(S3, 2),
        //             end_pitch: guitar.to_absulate_note_pitch(S3, 8),
        //             duration: Quarter,
        //         });

        //         // 休止停顿
        //         player.play_rest(Rest::new(Quarter));

        //         player.play_slide(SlideNote {
        //             start_pitch: guitar.to_absulate_note_pitch(S2, 2),
        //             end_pitch: guitar.to_absulate_note_pitch(S2, 10),
        //             duration: Quarter,
        //         });
        //         player.play_slide(SlideNote {
        //             start_pitch: guitar.to_absulate_note_pitch(S2, 16),
        //             end_pitch: guitar.to_absulate_note_pitch(S2, 0),
        //             duration: Quarter,
        //         });
        //         player.play_rest(Rest::new(Half));
        //     }

        //     //     guitar.set_capo_fret(20);
        //     //     player.set_beat_duration_from_bpm(240, Quarter);
        //     //     player.play_note(guitar.to_absulate_note(S1, 0, Sixteenth));
        //     //     player.play_rest(Rest::new(Sixteenth));
        //     //     player.play_note(guitar.to_absulate_note(S1, 0, Sixteenth));
        //     //     player.play_rest(Rest::new(Sixteenth));
        //     //     player.play_note(guitar.to_absulate_note(S1, 0, Sixteenth));
        //     //     player.play_rest(Rest::new(Sixteenth));
        //     //     player.play_note(guitar.to_absulate_note(S1, 0, Sixteenth));
        //     //     player.play_rest(Rest::new(Sixteenth));
        //     //     player.play_rest(Rest::new(HalfDotted));

        //     //     player.set_beat_duration_from_bpm(60, Quarter);
        //     //     player.play_note(guitar.to_absulate_note(S5, 0, Eighth));
        //     //     player.play_note(guitar.to_absulate_note(S3, 0, Eighth));
        //     //     player.play_note(guitar.to_absulate_note(S2, 0, Eighth));
        //     //     player.play_note(guitar.to_absulate_note(S3, 0, Eighth));
        //     //     player.play_note(guitar.to_absulate_note(S1, 0, Eighth));
        //     //     player.play_note(guitar.to_absulate_note(S3, 0, Eighth));
        //     //     player.play_note(guitar.to_absulate_note(S2, 0, Eighth));
        //     //     player.play_note(guitar.to_absulate_note(S3, 0, Eighth));

        //     //     for i in 3..12 {
        //     //         guitar.set_capo_fret(i);
        //     //         player.set_beat_duration_from_bpm(100, Quarter);

        //     //         player.play_rest(Rest::new(Quarter));
        //     //         player.play_slide(SlideNote {
        //     //             start_pitch: guitar.to_absulate_note_pitch(S4, 2),
        //     //             end_pitch: guitar.to_absulate_note_pitch(S4, 8),
        //     //             duration: Quarter,
        //     //         });
        //     //         player.play_note(guitar.to_absulate_note(S2, 7, Quarter));
        //     //         player.play_note(guitar.to_absulate_note(S2, 9, Quarter));
        //     //         player.play_note(guitar.to_absulate_note(S2, 11, Half));

        //     //         player.play_note(guitar.to_absulate_note(S2, 12, Eighth));
        //     //         player.play_note(guitar.to_absulate_note(S2, 11, Eighth));
        //     //         player.play_note(guitar.to_absulate_note(S2, 9, Half));

        //     //         player.play_note(guitar.to_absulate_note(S2, 9, Eighth));
        //     //         player.play_note(guitar.to_absulate_note(S2, 11, Eighth));
        //     //         player.play_note(guitar.to_absulate_note(S2, 9, Quarter));
        //     //         player.play_note(guitar.to_absulate_note(S2, 7, Quarter));
        //     //         player.play_note(guitar.to_absulate_note(S2, 6, Eighth));
        //     //         player.play_note(guitar.to_absulate_note(S2, 7, Eighth));

        //     //         player.play_note(guitar.to_absulate_note(S2, 6, Quarter));
        //     //         player.play_note(guitar.to_absulate_note(S2, 4, Half));

        //     //         player.play_note(guitar.to_absulate_note(S2, 7, Quarter));
        //     //         player.play_note(guitar.to_absulate_note(S2, 9, Quarter));
        //     //         player.play_note(guitar.to_absulate_note(S2, 11, Half));

        //     //         player.play_note(guitar.to_absulate_note(S2, 9, Quarter));
        //     //         player.play_note(guitar.to_absulate_note(S2, 7, Eighth));
        //     //         player.play_note(guitar.to_absulate_note(S2, 9, Half));

        //     //         player.play_note(guitar.to_absulate_note(S2, 7, Quarter));
        //     //         player.play_note(guitar.to_absulate_note(S2, 6, Quarter));
        //     //         player.play_note(guitar.to_absulate_note(S2, 7, Half));

        //     //         player.play_note(guitar.to_absulate_note(S2, 14, Quarter));
        //     //         player.play_note(guitar.to_absulate_note(S2, 14, Quarter));
        //     //     }
        // });
    }
}
