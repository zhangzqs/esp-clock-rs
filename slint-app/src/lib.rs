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

pub struct MyAppDeps<CONN, ConnErr, SYS, EGC, EGD, EGE>
where
    CONN: Connection<Error = ConnErr> + 'static + Send,
    ConnErr: error::Error + 'static,
    SYS: System + 'static,
    EGC: PixelColor + 'static + From<Rgb888>,
    EGD: DrawTarget<Color = EGC, Error = EGE> + 'static + Send,
    EGE: Debug + 'static,
{
    pub http_conn: CONN,
    pub system: SYS,
    pub display_group: Arc<Mutex<DisplayGroup<EGC, EGD>>>,
}

pub struct MyApp<CONN, ConnErr, SYS, EGC, EGD, EGE>
where
    CONN: Connection<Error = ConnErr> + 'static + Send,
    ConnErr: error::Error + 'static,
    EGC: PixelColor + 'static + From<Rgb888>,
    EGD: DrawTarget<Color = EGC, Error = EGE> + 'static + Send,
    EGE: Debug,
{
    app_window: AppWindow,
    home_time_timer: slint::Timer,
    system: SYS,
    http_client: Arc<Mutex<Client<CONN>>>, // 这个需要多线程传递共享
    photo_app: Rc<RefCell<PhotoApp<CONN, ConnErr, EGC, EGD>>>,
    clock_app: Rc<RefCell<ClockApp<EGC, EGD, EGE>>>,
    fpstest_app: Rc<RefCell<FPSTestApp<EGC, EGD, EGE>>>,
}

impl<CONN, ConnErr, SYS, EGC, EGD, EGE> MyApp<CONN, ConnErr, SYS, EGC, EGD, EGE>
where
    CONN: Connection<Error = ConnErr> + 'static + Send,
    ConnErr: error::Error + 'static,
    SYS: System + 'static,
    EGC: PixelColor + 'static + From<Rgb888>,
    EGD: DrawTarget<Color = EGC, Error = EGE> + 'static + Send,
    EGE: Debug + 'static,
{
    pub fn new(deps: MyAppDeps<CONN, ConnErr, SYS, EGC, EGD, EGE>) -> Self {
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
        let app = MyApp {
            home_time_timer: Self::start_home_time_timer(app_window.as_weak()),
            http_client,
            app_window,
            system: deps.system,
            photo_app,
            clock_app,
            fpstest_app,
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
