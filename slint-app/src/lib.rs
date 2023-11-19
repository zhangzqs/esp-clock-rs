use embedded_svc::http::{
    client::{Client, Connection},
    Method,
};
use log::info;
use slint::{Weak};
use std::{
    sync::{Arc, Mutex},
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
    _home_time_timer: slint::Timer,
    _http_client: Arc<Mutex<force_send_sync::Send<Client<C>>>>,
}

impl<C> MyApp<C>
where
    C: Connection + 'static,
{
    pub fn new(deps: MyAppDeps<C>) -> MyApp<C> {
        let app_window = AppWindow::new().expect("Failed to create AppWindow");
        MyApp {
            _home_time_timer: Self::start_home_time_timer(app_window.as_weak()),
            _http_client: Arc::new(Mutex::new(unsafe {
                force_send_sync::Send::new(Client::wrap(deps.http_conn))
            })),
            app_window,
        }
    }
    
    fn start_home_time_timer(w: Weak<AppWindow>) ->slint::Timer {
        let t = slint::Timer::default();
        t.start(
            slint::TimerMode::Repeated,
            Duration::from_secs(1),
            move || {
                let t = OffsetDateTime::now_utc()
                    .to_offset(UtcOffset::from_hms(8, 0, 0).unwrap());
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
        let c = self._http_client.clone();
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

    pub fn update_ui(&self, f: impl FnOnce(AppWindow)) {
        if let Some(ui) = self.app_window.as_weak().upgrade() {
            f(ui);
        }
    }

    pub fn set_boot_state(&self, state: BootState) {
        info!("set_boot_state {:?}", state);
        self.update_ui(|ui| ui.invoke_set_boot_state(state));
    }

    pub fn on_one_button_clicks(&self, clicks: i32) {
        info!("on_one_button_click");
        self.update_ui(|ui| ui.invoke_on_one_button_clicks(clicks));
    }

    pub fn on_one_button_long_pressed_holding(&self, dur: Duration) {
        info!("on_one_button_long_pressed_holding_time");
        self.update_ui(|ui| ui.invoke_on_one_button_long_pressed_holding(dur.as_millis() as _));
    }

    pub fn on_one_button_long_pressed_held(&self, dur: Duration) {
        info!("on_one_button_long_pressed_held_time");
        self.update_ui(|ui| ui.invoke_on_one_button_long_pressed_held(dur.as_millis() as _));
    }
}
