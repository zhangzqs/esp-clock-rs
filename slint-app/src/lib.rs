use embedded_svc::http::{
    client::{Client, Connection},
    Method,
};
use force_send_sync::Send;
use log::info;
use slint::Weak;
use std::sync::{mpsc, Arc, Mutex};
use std::{thread, time::Duration};
use time::{OffsetDateTime, UtcOffset};

slint::include_modules!();

pub struct MyAppDeps<C>
where
    C: Connection + 'static,
{
    pub http_conn: C,
}

pub struct MyApp<C> {
    app_window: AppWindow,
    timer: slint::Timer,
    page_timer: slint::Timer,
    http_client: Arc<Mutex<Send<Client<C>>>>,
}

impl<C> MyApp<C>
where
    C: Connection + 'static,
{
    pub fn get_app_window_as_weak(&self) -> Weak<AppWindow> {
        self.app_window.as_weak()
    }

    pub fn new(deps: MyAppDeps<C>) -> MyApp<C> {
        let app_window = AppWindow::new().expect("Failed to create AppWindow");
        let app = MyApp {
            timer: {
                let t = slint::Timer::default();
                let u = app_window.as_weak();
                t.start(
                    slint::TimerMode::Repeated,
                    Duration::from_secs(1),
                    move || {
                        let t = OffsetDateTime::now_utc()
                            .to_offset(UtcOffset::from_hms(8, 0, 0).unwrap());
                        let dt = DateTime {
                            day: t.day() as i32,
                            hour: t.hour() as i32,
                            minute: t.minute() as i32,
                            month: t.month() as i32,
                            second: t.second() as i32,
                            week: t.weekday().number_days_from_sunday() as i32,
                            year: t.year() as i32,
                        };
                        if let Some(ui) = u.upgrade() {
                            ui.set_date_time(dt);
                        }
                    },
                );
                t
            },
            page_timer: {
                let t = slint::Timer::default();
                let u = app_window.as_weak();
                t.start(
                    slint::TimerMode::Repeated,
                    Duration::from_secs(5),
                    move || {
                        if let Some(ui) = u.upgrade() {
                            // 还在boot界面，不自动翻页
                            if ui.get_page_id() == -1 {
                                return;
                            }
                            let s = ui.get_page_size();
                            let p = ui.get_page_id();
                            ui.set_page_id((p + 1) % s);
                        }
                    },
                );
                t
            },
            app_window: app_window,
            http_client: Arc::new(Mutex::new(unsafe {
                Send::new(Client::wrap(deps.http_conn))
            })),
        };
        app
    }

    pub fn update_ip(&self) {
        println!("update_ip");
        let c = self.http_client.clone();
        let u = self.app_window.as_weak();
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
            let mut buf_read = resp.read(&mut buf).unwrap();
            let ip = std::str::from_utf8(&buf[..buf_read]).unwrap().trim();
            println!("got ip: {}", ip);
            if let Some(ui) = u.upgrade() {
                ui.set_ip(ip.into());
            }
        });
    }

    pub fn go_to_next_page(&self) {
        info!("go_to_next_page");
        self.page_timer.restart();
        self.update_ip();

        let u = self.app_window.as_weak();
        if let Some(ui) = u.upgrade() {
            let s = ui.get_page_size();
            let p = ui.get_page_id();
            ui.set_page_id((p + 1) % s);
        }
    }

    pub fn go_to_prev_page(&self) {
        info!("go_to_prev_page");
        self.page_timer.restart();

        let u = self.app_window.as_weak();
        if let Some(ui) = u.upgrade() {
            let s = ui.get_page_size();
            let p = ui.get_page_id();
            ui.set_page_id((p + s - 1) % s);
        }
    }

    pub fn go_to_home_page(&self) {
        info!("go_to_home_page");
        self.page_timer.restart();

        let u = self.app_window.as_weak();
        if let Some(ui) = u.upgrade() {
            ui.set_page_id(0);
        }
    }
}
