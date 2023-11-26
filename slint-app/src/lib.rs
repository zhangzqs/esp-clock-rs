use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::{PixelColor, Rgb888},
};
use embedded_graphics_group::DisplayGroup;
use embedded_svc::{
    http::{
        client::{Client, Connection},
        Method,
    },
    io::Read,
};
use embedded_tone::{AbsulateNotePitch, Player, SlideNote};
use log::{debug, info};
use slint::Weak;
use std::{
    cell::RefCell,
    error,
    fmt::Debug,
    rc::Rc,
    sync::{Arc, Mutex},
};
use std::{thread, time::Duration};
use time::{OffsetDateTime, UtcOffset};

mod system;
pub use system::System;

use crate::photo::PhotoApp;
mod photo;

mod clock;
use crate::clock::ClockApp;

mod fpstest;
use crate::fpstest::FPSTestApp;
mod hsv;

slint::include_modules!();

pub struct MyAppDeps<CONN, ConnErr, SYS, EGC, EGD, EGE, TONE>
where
    CONN: Connection<Error = ConnErr> + 'static + Send,
    ConnErr: error::Error + 'static,
    SYS: System + 'static,
    EGC: PixelColor + 'static + From<Rgb888>,
    EGD: DrawTarget<Color = EGC, Error = EGE> + 'static + Send,
    EGE: Debug + 'static,
    TONE: Player + 'static + Send,
{
    pub http_conn: CONN,
    pub system: SYS,
    pub display_group: Arc<Mutex<DisplayGroup<EGC, EGD>>>,
    pub player: TONE,
}

pub struct MyApp<CONN, ConnErr, SYS, EGC, EGD, EGE, TONE>
where
    CONN: Connection<Error = ConnErr> + 'static + Send,
    ConnErr: error::Error + 'static,
    EGC: PixelColor + 'static + From<Rgb888>,
    EGD: DrawTarget<Color = EGC, Error = EGE> + 'static + Send,
    EGE: Debug,
    TONE: Player + 'static + Send,
{
    app_window: AppWindow,
    home_time_timer: slint::Timer,
    system: SYS,
    http_client: Arc<Mutex<Client<CONN>>>, // 这个需要多线程传递共享
    photo_app: Rc<RefCell<PhotoApp<CONN, ConnErr, EGC, EGD>>>,
    clock_app: Rc<RefCell<ClockApp<EGC, EGD, EGE>>>,
    fpstest_app: Rc<RefCell<FPSTestApp<EGC, EGD, EGE>>>,
    player: Arc<Mutex<TONE>>,
}

impl<CONN, ConnErr, SYS, EGC, EGD, EGE, TONE> MyApp<CONN, ConnErr, SYS, EGC, EGD, EGE, TONE>
where
    CONN: Connection<Error = ConnErr> + 'static + Send,
    ConnErr: error::Error + 'static,
    SYS: System + 'static,
    EGC: PixelColor + 'static + From<Rgb888>,
    EGD: DrawTarget<Color = EGC, Error = EGE> + 'static + Send,
    EGE: Debug + 'static,
    TONE: Player + 'static + Send,
{
    pub fn new(deps: MyAppDeps<CONN, ConnErr, SYS, EGC, EGD, EGE, TONE>) -> Self {
        debug!("MyApp::new");
        let app_window = AppWindow::new().expect("Failed to create AppWindow");
        debug!("AppWindow created");
        let http_client = Arc::new(Mutex::new(Client::wrap(deps.http_conn)));
        debug!("HttpClient created");
        let photo_app = Rc::new(RefCell::new(PhotoApp::new(
            http_client.clone(),
            deps.display_group.clone(),
        )));
        let clock_app = Rc::new(RefCell::new(ClockApp::new(deps.display_group.clone())));
        let fpstest_app = Rc::new(RefCell::new(FPSTestApp::new(deps.display_group.clone())));
        let player = Arc::new(Mutex::new(deps.player));

        let player_ref = player.clone();
        thread::spawn(move || {
            let mut player = player_ref.lock().unwrap();
            use embedded_tone::{
                Guitar,
                GuitarString::*,
                NoteDuration::{Eighth, Half, HalfDotted, Quarter, Sixteenth},
                NoteName::*,
                Octave::*,
                Rest,
            };

            let mut guitar = Guitar::default();

            for i in (4..12).step_by(2) {
                guitar.set_capo_fret(i);
                player.set_beat_duration_from_bpm(120, Quarter);
    
                player.play_slide(SlideNote {
                    start_pitch: guitar.to_absulate_note_pitch(S3, 2),
                    end_pitch: guitar.to_absulate_note_pitch(S3, 8),
                    duration: Quarter,
                });
                player.play_slide(SlideNote {
                    start_pitch: guitar.to_absulate_note_pitch(S3, 2),
                    end_pitch: guitar.to_absulate_note_pitch(S3, 8),
                    duration: Quarter,
                });
    
                // 休止停顿
                player.play_rest(Rest::new(Quarter));
    
                player.play_slide(SlideNote {
                    start_pitch: guitar.to_absulate_note_pitch(S2, 2),
                    end_pitch: guitar.to_absulate_note_pitch(S2, 10),
                    duration: Quarter,
                });
                player.play_slide(SlideNote {
                    start_pitch: guitar.to_absulate_note_pitch(S2, 16),
                    end_pitch: guitar.to_absulate_note_pitch(S2, 0),
                    duration: Quarter,
                });
                player.play_rest(Rest::new(Half));
            }


            //     guitar.set_capo_fret(20);
            //     player.set_beat_duration_from_bpm(240, Quarter);
            //     player.play_note(guitar.to_absulate_note(S1, 0, Sixteenth));
            //     player.play_rest(Rest::new(Sixteenth));
            //     player.play_note(guitar.to_absulate_note(S1, 0, Sixteenth));
            //     player.play_rest(Rest::new(Sixteenth));
            //     player.play_note(guitar.to_absulate_note(S1, 0, Sixteenth));
            //     player.play_rest(Rest::new(Sixteenth));
            //     player.play_note(guitar.to_absulate_note(S1, 0, Sixteenth));
            //     player.play_rest(Rest::new(Sixteenth));
            //     player.play_rest(Rest::new(HalfDotted));

            //     player.set_beat_duration_from_bpm(60, Quarter);
            //     player.play_note(guitar.to_absulate_note(S5, 0, Eighth));
            //     player.play_note(guitar.to_absulate_note(S3, 0, Eighth));
            //     player.play_note(guitar.to_absulate_note(S2, 0, Eighth));
            //     player.play_note(guitar.to_absulate_note(S3, 0, Eighth));
            //     player.play_note(guitar.to_absulate_note(S1, 0, Eighth));
            //     player.play_note(guitar.to_absulate_note(S3, 0, Eighth));
            //     player.play_note(guitar.to_absulate_note(S2, 0, Eighth));
            //     player.play_note(guitar.to_absulate_note(S3, 0, Eighth));

            //     for i in 3..12 {
            //         guitar.set_capo_fret(i);
            //         player.set_beat_duration_from_bpm(100, Quarter);

            //         player.play_rest(Rest::new(Quarter));
            //         player.play_slide(SlideNote {
            //             start_pitch: guitar.to_absulate_note_pitch(S4, 2),
            //             end_pitch: guitar.to_absulate_note_pitch(S4, 8),
            //             duration: Quarter,
            //         });
            //         player.play_note(guitar.to_absulate_note(S2, 7, Quarter));
            //         player.play_note(guitar.to_absulate_note(S2, 9, Quarter));
            //         player.play_note(guitar.to_absulate_note(S2, 11, Half));

            //         player.play_note(guitar.to_absulate_note(S2, 12, Eighth));
            //         player.play_note(guitar.to_absulate_note(S2, 11, Eighth));
            //         player.play_note(guitar.to_absulate_note(S2, 9, Half));

            //         player.play_note(guitar.to_absulate_note(S2, 9, Eighth));
            //         player.play_note(guitar.to_absulate_note(S2, 11, Eighth));
            //         player.play_note(guitar.to_absulate_note(S2, 9, Quarter));
            //         player.play_note(guitar.to_absulate_note(S2, 7, Quarter));
            //         player.play_note(guitar.to_absulate_note(S2, 6, Eighth));
            //         player.play_note(guitar.to_absulate_note(S2, 7, Eighth));

            //         player.play_note(guitar.to_absulate_note(S2, 6, Quarter));
            //         player.play_note(guitar.to_absulate_note(S2, 4, Half));

            //         player.play_note(guitar.to_absulate_note(S2, 7, Quarter));
            //         player.play_note(guitar.to_absulate_note(S2, 9, Quarter));
            //         player.play_note(guitar.to_absulate_note(S2, 11, Half));

            //         player.play_note(guitar.to_absulate_note(S2, 9, Quarter));
            //         player.play_note(guitar.to_absulate_note(S2, 7, Eighth));
            //         player.play_note(guitar.to_absulate_note(S2, 9, Half));

            //         player.play_note(guitar.to_absulate_note(S2, 7, Quarter));
            //         player.play_note(guitar.to_absulate_note(S2, 6, Quarter));
            //         player.play_note(guitar.to_absulate_note(S2, 7, Half));

            //         player.play_note(guitar.to_absulate_note(S2, 14, Quarter));
            //         player.play_note(guitar.to_absulate_note(S2, 14, Quarter));
            //     }
        });

        let app = MyApp {
            home_time_timer: Self::start_home_time_timer(app_window.as_weak()),
            http_client,
            app_window,
            system: deps.system,
            photo_app,
            clock_app,
            fpstest_app,
            player,
        };
        info!("MyApp created");
        app.bind_event_app();

        app
    }

    fn bind_event_app(&self) {
        info!("bind_event_photo_app");
        if let Some(ui) = self.app_window.as_weak().upgrade() {
            let photo_app = self.photo_app.clone();
            ui.on_photo_page_enter(move || {
                info!("on_photo_page_enter");
                photo_app.borrow_mut().enter();
            });
            let photo_app = self.photo_app.clone();

            ui.on_photo_page_exit(move || {
                info!("on_photo_page_exit");
                photo_app.borrow_mut().exit();
            });
            let photo_app = self.photo_app.clone();
            ui.on_photo_page_request_next(move || {
                info!("on_photo_page_request_next");
                photo_app.borrow_mut().next();
            });
            let photo_app = self.photo_app.clone();
            ui.on_photo_page_request_auto_play(move || {
                info!("on_photo_page_request_auto_play");
                photo_app.borrow_mut().auto_play();
            });
            let photo_app = self.photo_app.clone();
            ui.on_photo_page_request_stop_auto_play(move || {
                info!("on_photo_page_request_stop_auto_play");
                photo_app.borrow_mut().stop_auto_play();
            });
            let clock_app = self.clock_app.clone();
            ui.on_clock_page_enter(move || {
                info!("on_clock_page_enter");
                clock_app.borrow_mut().enter();
            });
            let clock_app = self.clock_app.clone();
            ui.on_clock_page_exit(move || {
                info!("on_clock_page_exit");
                clock_app.borrow_mut().exit();
            });
            let fpstest_app = self.fpstest_app.clone();
            ui.on_fpstest_page_enter(move || {
                info!("on_fpstest_page_enter");
                fpstest_app.borrow_mut().enter();
            });
            let fpstest_app = self.fpstest_app.clone();
            ui.on_fpstest_page_exit(move || {
                info!("on_fpstest_page_exit");
                fpstest_app.borrow_mut().exit();
            });
            let fpstest_app = self.fpstest_app.clone();
            ui.on_fpstest_page_update_type(move |t| {
                info!("on_fpstest_page_update_type");
                fpstest_app.borrow_mut().update_type(t);
            });
        }
    }

    fn start_home_time_timer(w: Weak<AppWindow>) -> slint::Timer {
        let t = slint::Timer::default();
        t.start(
            slint::TimerMode::Repeated,
            Duration::from_secs(1),
            move || {
                let t = OffsetDateTime::now_utc().to_offset(UtcOffset::from_hms(8, 0, 0).unwrap());
                if let Some(ui) = w.upgrade() {
                    ui.set_home_page_time(HomeTimeData {
                        day: t.day() as i32,
                        hour: t.hour() as i32,
                        minute: t.minute() as i32,
                        month: t.month() as i32,
                        second: t.second() as i32,
                        week: t.weekday().number_days_from_sunday() as i32,
                        year: t.year(),
                    });
                }
            },
        );
        t
    }

    fn _update_ip(&self) {
        println!("update_ip");
        let c = self.http_client.clone();
        let _u = self.app_window.as_weak();
        thread::spawn(move || {
            let mut client = c.lock().unwrap();
            let req = client
                .request(
                    Method::Get,
                    "http://ifconfig.net/",
                    &[("accept", "text/plain")],
                )
                .unwrap();
            let mut resp = req.submit().unwrap();
            let mut buf = [0u8; 30];
            let buf_read = resp.read(&mut buf).unwrap();
            let ip = std::str::from_utf8(&buf[..buf_read]).unwrap().trim();
            println!("got ip: {}", ip);
        });
    }

    pub fn run(&self) -> Result<(), slint::PlatformError> {
        slint::run_event_loop()?;
        Ok(())
    }

    pub fn get_app_window(&self) -> Weak<AppWindow> {
        self.app_window.as_weak()
    }
}
