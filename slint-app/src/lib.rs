use embedded_svc::http::{
    client::{Client, Connection},
    Method,
};
use force_send_sync::Send;
use log::info;
use slint::{Image, SharedPixelBuffer, Weak, Rgb8Pixel};
use std::{
    io::{BufReader, Bytes, Cursor},
    sync::{mpsc, Arc, Mutex},
};
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
                        let dt = HomeData {
                            day: t.day() as i32,
                            hour: t.hour() as i32,
                            minute: t.minute() as i32,
                            month: t.month() as i32,
                            second: t.second() as i32,
                            week: t.weekday().number_days_from_sunday() as i32,
                            year: t.year() as i32,
                            current_humi: 1,
                            current_temp: 2,
                            today_weather_desc: Weather::Cloudy,
                            today_weather_location: Location::Shanghai,
                            today_weather_max_temp: 3,
                            today_weather_min_temp: 4,
                        };
                        if let Some(ui) = u.upgrade() {
                            ui.set_home_data(dt);
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
        });
    }

    pub fn set_boot_state(&self, state: BootState) {
        info!("set_boot_state {:?}", state);
        let u = self.get_app_window_as_weak();
        if let Some(ui) = u.upgrade() {
            ui.invoke_set_boot_state(state);
        }
    }

    pub fn on_one_button_clicks(&self, clicks: i32) {
        info!("on_one_button_click");
        let u = self.get_app_window_as_weak();
        if let Some(ui) = u.upgrade() {
            ui.invoke_on_one_button_clicks(clicks);
        }
    }

    pub fn on_one_button_long_pressed_holding_time(&self, dur: Duration) {
        info!("on_one_button_long_pressed_holding_time");
        let u = self.get_app_window_as_weak();
        if let Some(ui) = u.upgrade() {
            ui.invoke_on_one_button_long_pressed_holding_time(dur.as_millis() as _);
        }
    }

    pub fn on_one_button_long_pressed_held_time(&self, dur: Duration) {
        info!("on_one_button_long_pressed_held_time");
        let u = self.get_app_window_as_weak();
        if let Some(ui) = u.upgrade() {
            ui.invoke_on_one_button_long_pressed_held_time(dur.as_millis() as _);
        }
    }
}
