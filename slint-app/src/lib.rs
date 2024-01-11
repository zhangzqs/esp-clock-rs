use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::{PixelColor, Rgb888},
};
use embedded_graphics_group::DisplayGroup;

use embedded_svc::storage::RawStorage;
use embedded_tone::RawTonePlayer;
use log::{debug, info};
use slint::Weak;
use std::{
    cell::RefCell,
    fmt::Debug,
    marker::PhantomData,
    rc::Rc,
    sync::{Arc, Mutex},
};

mod app;
mod interface;
mod storage;
mod util;

use app::*;

// 公开一些接口
pub use app::EvilApple;
pub use interface::*;

use crate::storage::{Storage, StorageMut};

slint::include_modules!();

pub struct MyAppDeps<CB, SB, SYS, EGC, EGD, EGE, TONE, EA, SCBC, LC, RS>
where
    SYS: System + 'static,
    EGC: PixelColor + 'static + From<Rgb888>,
    EGD: DrawTarget<Color = EGC, Error = EGE> + 'static + Send,
    EGE: Debug + 'static,
    TONE: RawTonePlayer + 'static + Send,
    EA: EvilApple + 'static,
    LC: LEDController + 'static + Send,
    SCBC: LEDController + 'static + Send,
    SB: ServerBuilder<'static>,
    CB: ClientBuilder + 'static,
    RS: RawStorage,
{
    pub system: SYS,
    pub display_group: Arc<Mutex<DisplayGroup<EGD>>>,
    pub player: TONE,
    pub eval_apple: EA,
    pub screen_brightness_controller: SCBC,
    pub blue_led: LC,
    pub http_server_builder: PhantomData<SB>,
    pub http_client_builder: PhantomData<CB>,
    pub raw_storage: RS,
}

pub struct MyApp<CB, SB, SYS, EGC, EGD, EGE, TONE, EA, SCBC, LC, RS>
where
    EGC: PixelColor + 'static + From<Rgb888>,
    EGD: DrawTarget<Color = EGC, Error = EGE> + 'static + Send,
    EGE: Debug,
    TONE: RawTonePlayer + 'static + Send,
    EA: EvilApple,
    LC: LEDController + 'static + Send,
    SCBC: LEDController + 'static + Send,
    SB: ServerBuilder<'static>,
    CB: ClientBuilder + 'static,
    RS: RawStorage,
{
    app_window: AppWindow,
    system: SYS,
    photo_app: Rc<RefCell<PhotoApp<CB, EGC, EGD>>>,
    clock_app: Rc<RefCell<ClockApp<EGC, EGD, EGE>>>,
    fpstest_app: Rc<RefCell<FPSTestApp<EGC, EGD, EGE>>>,
    projector_app: Rc<RefCell<ProjectorApp<EGC, EGD, EGE>>>,
    evil_apple_app: Rc<RefCell<EvilAppleApp<EA>>>,
    music_app: Rc<RefCell<MusicApp<TONE, LC>>>,
    _screen_led_ctl: Arc<Mutex<SCBC>>,
    home_app: Rc<RefCell<HomeApp<CB>>>,
    network_monitor_app: Rc<RefCell<NetworkMonitorApp>>,
    http_server_app: Rc<RefCell<HttpServerApp<SB, SCBC>>>,
    raw_storage: RS,
}

impl<CB, SB, SYS, EGC, EGD, EGE, TONE, EA, SCBC, LC, RS>
    MyApp<CB, SB, SYS, EGC, EGD, EGE, TONE, EA, SCBC, LC, RS>
where
    SYS: System + 'static,
    EGC: PixelColor + 'static + From<Rgb888>,
    EGD: DrawTarget<Color = EGC, Error = EGE> + 'static + Send,
    EGE: Debug + 'static,
    TONE: RawTonePlayer + 'static + Send,
    EA: EvilApple,
    LC: LEDController + 'static + Send,
    SCBC: LEDController + 'static + Send,
    SB: ServerBuilder<'static>,
    CB: ClientBuilder,
    RS: RawStorage,
{
    pub fn new(deps: MyAppDeps<CB, SB, SYS, EGC, EGD, EGE, TONE, EA, SCBC, LC, RS>) -> Self {
        let mut raw_storage = deps.raw_storage;
        StorageMut(&mut raw_storage).system_mut().inc_boot_count();
        let cnt = Storage(&raw_storage).system().get_boot_count();
        info!("boot count: {}", cnt);
        let app_window = AppWindow::new().expect("Failed to create AppWindow");
        debug!("AppWindow created");
        let photo_app = Rc::new(RefCell::new(PhotoApp::new(deps.display_group.clone())));
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
        let network_monitor_app =
            Rc::new(RefCell::new(NetworkMonitorApp::new(app_window.as_weak())));

        let http_server_app = Rc::new(RefCell::new(HttpServerApp::new(
            app_window.as_weak(),
            screen_led_ctl.clone(),
        )));
        let app = MyApp {
            app_window,
            system: deps.system,
            photo_app,
            clock_app,
            fpstest_app,
            projector_app,
            evil_apple_app,
            music_app,
            home_app,
            network_monitor_app,
            http_server_app,
            raw_storage,
            _screen_led_ctl: screen_led_ctl,
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
            ui.on_music_page_play(move |i| {
                info!("on_music_page_switch: {:?}", i);
                music_app.borrow_mut().play(i)
            });
        }
    }

    // fn _update_ip(&self) {
    //     println!("update_ip");
    //     let c = self.http_client.clone();
    //     let _u = self.app_window.as_weak();
    //     thread::spawn(move || {
    //         let mut client = c.lock().unwrap();
    //         let req = client
    //             .request(
    //                 Method::Get,
    //                 "http://ifconfig.net/",
    //                 &[("accept", "text/plain")],
    //             )
    //             .unwrap();
    //         let mut resp = req.submit().unwrap();
    //         let mut buf = [0u8; 30];
    //         let buf_read = resp.read(&mut buf).unwrap();
    //         let ip = std::str::from_utf8(&buf[..buf_read]).unwrap().trim();
    //         println!("got ip: {}", ip);
    //     });
    // }

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
