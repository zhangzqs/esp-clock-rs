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
use embedded_tone::RawTonePlayer;
use home::HomeApp;
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

mod projector;
use crate::projector::ProjectorApp;

mod system;
pub use system::System;

use crate::photo::PhotoApp;
mod photo;

mod clock;
use crate::clock::ClockApp;

mod fpstest;
use crate::fpstest::FPSTestApp;
mod hsv;

mod evil_apple;
pub use crate::evil_apple::{EvilApple, EvilAppleApp};

pub use system::MockSystem;

mod led_controller;
pub use led_controller::LEDController;

mod music;
use crate::music::MusicApp;

mod home;

slint::include_modules!();

pub struct MyAppDeps<CONN, ConnErr, SYS, EGC, EGD, EGE, TONE, EA, LC>
where
    CONN: Connection<Error = ConnErr> + 'static + Send,
    ConnErr: error::Error + 'static,
    SYS: System + 'static,
    EGC: PixelColor + 'static + From<Rgb888>,
    EGD: DrawTarget<Color = EGC, Error = EGE> + 'static + Send,
    EGE: Debug + 'static,
    TONE: RawTonePlayer + 'static + Send,
    EA: EvilApple + 'static,
    LC: LEDController + 'static + Send,
{
    pub http_conn: CONN,
    pub system: SYS,
    pub display_group: Arc<Mutex<DisplayGroup<EGC, EGD>>>,
    pub player: TONE,
    pub eval_apple: EA,
    pub screen_brightness_controller: LC,
    pub blue_led: LC,
}

pub struct MyApp<CONN, ConnErr, SYS, EGC, EGD, EGE, TONE, EA, LC>
where
    CONN: Connection<Error = ConnErr> + 'static + Send,
    ConnErr: error::Error + 'static,
    EGC: PixelColor + 'static + From<Rgb888>,
    EGD: DrawTarget<Color = EGC, Error = EGE> + 'static + Send,
    EGE: Debug,
    TONE: RawTonePlayer + 'static + Send,
    EA: EvilApple,
    LC: LEDController + 'static + Send,
{
    app_window: AppWindow,
    system: SYS,
    http_client: Arc<Mutex<Client<CONN>>>, // 这个需要多线程传递共享
    photo_app: Rc<RefCell<PhotoApp<CONN, ConnErr, EGC, EGD>>>,
    clock_app: Rc<RefCell<ClockApp<EGC, EGD, EGE>>>,
    fpstest_app: Rc<RefCell<FPSTestApp<EGC, EGD, EGE>>>,
    projector_app: Rc<RefCell<ProjectorApp<EGC, EGD, EGE>>>,
    evil_apple_app: Rc<RefCell<EvilAppleApp<EA>>>,
    music_app: Rc<RefCell<MusicApp<TONE, LC>>>,
    _screen_led_ctl: Arc<Mutex<LC>>,
    home_app: Rc<RefCell<HomeApp>>,
}

impl<CONN, ConnErr, SYS, EGC, EGD, EGE, TONE, EA, LC>
    MyApp<CONN, ConnErr, SYS, EGC, EGD, EGE, TONE, EA, LC>
where
    CONN: Connection<Error = ConnErr> + 'static + Send,
    ConnErr: error::Error + 'static,
    SYS: System + 'static,
    EGC: PixelColor + 'static + From<Rgb888>,
    EGD: DrawTarget<Color = EGC, Error = EGE> + 'static + Send,
    EGE: Debug + 'static,
    TONE: RawTonePlayer + 'static + Send,
    EA: EvilApple,
    LC: LEDController + 'static + Send,
{
    pub fn new(deps: MyAppDeps<CONN, ConnErr, SYS, EGC, EGD, EGE, TONE, EA, LC>) -> Self {
        let app_window = AppWindow::new().expect("Failed to create AppWindow");
        debug!("AppWindow created");
        let http_client = Arc::new(Mutex::new(Client::wrap(deps.http_conn)));
        let photo_app = Rc::new(RefCell::new(PhotoApp::new(
            http_client.clone(),
            deps.display_group.clone(),
        )));
        let clock_app = Rc::new(RefCell::new(ClockApp::new(deps.display_group.clone())));
        let fpstest_app = Rc::new(RefCell::new(FPSTestApp::new(deps.display_group.clone())));
        let projector_app = Rc::new(RefCell::new(ProjectorApp::new(
            deps.display_group.clone(),
            app_window.as_weak(),
        )));
        let player = Arc::new(Mutex::new(deps.player));
        let evil_apple_app = Rc::new(RefCell::new(EvilAppleApp::new(deps.eval_apple)));
        let screen_led_ctl = Arc::new(Mutex::new(deps.screen_brightness_controller));
        let blue_led = Arc::new(Mutex::new(deps.blue_led));
        let music_app = Rc::new(RefCell::new(MusicApp::new(
            app_window.as_weak(),
            player.clone(),
            blue_led.clone(),
        )));
        let home_app = Rc::new(RefCell::new(HomeApp::new(app_window.as_weak())));
        let app = MyApp {
            http_client,
            app_window,
            system: deps.system,
            photo_app,
            clock_app,
            fpstest_app,
            projector_app,
            evil_apple_app,
            music_app,
            _screen_led_ctl: screen_led_ctl,
            home_app,
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
            let projector_app = self.projector_app.clone();
            ui.on_projector_page_enter(move || {
                info!("on_projector_page_enter");
                projector_app.borrow_mut().enter();
            });
            let projector_app = self.projector_app.clone();
            ui.on_projector_page_exit(move || {
                info!("on_projector_page_exit");
                projector_app.borrow_mut().exit();
            });
            let music_app = self.music_app.clone();
            ui.on_music_page_enter(move || {
                info!("on_music_page_enter");
                music_app.borrow_mut().enter();
            });
            let music_app = self.music_app.clone();
            ui.on_music_page_exit(move || {
                info!("on_music_page_exit");
                music_app.borrow_mut().exit();
            });
            let music_app = self.music_app.clone();
            ui.on_music_page_switch(move |i| {
                info!("on_music_page_switch: {:?}", i);
                music_app.borrow_mut().switch(i);
            });
        }
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
        slint::run_event_loop()
    }

    pub fn run_event_loop(&self) -> Result<(), slint::PlatformError> {
        Ok(())
    }

    pub fn get_app_window(&self) -> Weak<AppWindow> {
        self.app_window.as_weak()
    }
}
