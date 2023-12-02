use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use embedded_tone::{AbsulateNotePitch, RawTonePlayer};
use log::info;
use midly::Timing;
use slint::Weak;

use crate::{AppWindow, LEDController};

const DATA1: &'static [u8] = include_bytes!("../music/ql.mid");
const DATA2: &'static [u8] = include_bytes!("../music/fontaine.mid");
const DATA3: &'static [u8] = include_bytes!("../music/qqz.mid");
const DATA4: &'static [u8] = include_bytes!("../music/gy.mid");
const DATA5: &'static [u8] = include_bytes!("../music/wbcwj.mid");
const DATA6: &'static [u8] = include_bytes!("../music/Klee.mid");
const DATA7: &'static [u8] = include_bytes!("../music/nxd.mid");
const DATA8: &'static [u8] = include_bytes!("../music/ldjj.mid");
const DATA: &'static [u8] = include_bytes!("../music/yaoyao.mid");


pub struct MusicApp<TONE, LC>
where
    TONE: RawTonePlayer + 'static + Send,
    LC: LEDController + 'static + Send,
{
    app: Weak<AppWindow>,
    tone_player: Arc<Mutex<TONE>>,
    led: Arc<Mutex<LC>>,

    exit_signal: Arc<Mutex<bool>>,
    join_handle: Option<thread::JoinHandle<()>>,
}

impl<TONE, LC> MusicApp<TONE, LC>
where
    TONE: RawTonePlayer + 'static + Send,
    LC: LEDController + 'static + Send,
{
    pub fn new(app: Weak<AppWindow>, tone_player: Arc<Mutex<TONE>>, led: Arc<Mutex<LC>>) -> Self {
        Self {
            app,
            tone_player,
            led,
            exit_signal: Arc::new(Mutex::new(false)),
            join_handle: None,
        }
    }

    pub fn enter(&mut self) {
        let tone_player = self.tone_player.clone();
        let led = self.led.clone();
        let exit_signal_ref = self.exit_signal.clone();
        let app = self.app.clone();

        self.join_handle = Some(thread::spawn(move || {
            let mut player = tone_player.lock().unwrap();
            let mut led = led.lock().unwrap();
            *exit_signal_ref.lock().unwrap() = false;

            let (header, mut tracks) = midly::parse(&DATA).unwrap();
            // let 
            // if let Timing::Metrical(x) = header.timing {
            //     x.as_int()*4
            // }
            let track = tracks.next().unwrap().unwrap();
            let mut max_freq = 0;
            let mut min_freq = 10000;

            for event in track {
                if *exit_signal_ref.lock().unwrap() {
                    break;
                }
                if let Ok(event) = event {
                    thread::sleep(Duration::from_millis(event.delta.as_int() as u64));
                    if let Some(e) = event.kind.as_live_event() {
                        match e {
                            midly::live::LiveEvent::Midi { channel, message } => match message {
                                midly::MidiMessage::NoteOff { key, vel } => {
                                    println!("off");
                                    player.off();
                                }
                                midly::MidiMessage::NoteOn { key, vel } => {
                                    println!("key: {}, vel: {}", key, vel);
                                    let p = AbsulateNotePitch::from_midi_note_key(key.as_int());
                                    let freq = p.frequency();
                                    println!("freq: {}", freq);
                                    max_freq = max_freq.max(freq);
                                    min_freq = min_freq.min(freq);
                                    let s = (freq - min_freq) as f32 / (max_freq - min_freq) as f32;
                                    println!("screen: {}", s);
                                    led.set_brightness_percent(s);
                                    app.upgrade_in_event_loop(move |ui| {
                                        ui.set_music_page_note(format!("{:?}", p).into());
                                        ui.set_music_page_percent(s);
                                    })
                                    .unwrap();
                                    player.tone(freq);
                                }
                                _ => (),
                            },
                            _ => (),
                        }
                    }
                }
            }
            player.off();
        }));
    }
    pub fn exit(&mut self) {
        info!("recv exit signal");
        if self.join_handle.is_none() {
            return;
        }
        *self.exit_signal.lock().unwrap() = true;
        info!("music exit");
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
