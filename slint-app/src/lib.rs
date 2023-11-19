use embedded_svc::{
    http::{
        client::{Client, Connection},
        Method,
    },
    io::Read,
};
use log::{debug, info};
use slint::{Image, Rgb8Pixel, SharedPixelBuffer, Weak};
use std::{
    panic,
    sync::{Arc, Mutex},
    vec,
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
        let app = MyApp {
            _home_time_timer: Self::start_home_time_timer(app_window.as_weak()),
            _http_client: Arc::new(Mutex::new(unsafe {
                force_send_sync::Send::new(Client::wrap(deps.http_conn))
            })),
            app_window,
        };
        app.bind_event_on_photo_page_request_next();
        app.bind_event_on_photo_page_request_auto_play();
        app
    }

    fn bind_event_on_photo_page_request_auto_play(&self) {
        info!("bind_event_on_photo_page_request_auto_play");
        if let Some(ui) = self.app_window.as_weak().upgrade() {
            let c = self._http_client.clone();
            let u = ui.as_weak();
            ui.on_photo_page_request_auto_play(move || {
                info!("on_photo_page_request_auto_play");
                if let Some(ui) = u.upgrade() {
                    ui.set_photo_page_source(Default::default());
                }
                let c = c.clone();
                let u = u.clone();
                thread::spawn(move || {
                    let mut client = c.lock().unwrap();
                    loop {
                        if let Some(ui) = u.upgrade() {
                            if !ui.invoke_photo_page_is_auto_play_mode() {
                                break;
                            }
                        }
                        let req = client
                            .request(Method::Get, "http://192.168.242.118:3000/api/photo", &[])
                            .unwrap();
                        let mut resp = req.submit().unwrap();
                        let mut byte_buf = [0u8; 2];
                        resp.read_exact(&mut byte_buf).unwrap();
                        let width = byte_buf[0] as u32;
                        let height = byte_buf[1] as u32;
                        info!("read frame: {}x{}", width, height);

                        // 这一步可能会内存分配失败
                        let buf = panic::catch_unwind(move || {
                            SharedPixelBuffer::<Rgb8Pixel>::new(width, height)
                        });
                        if buf.is_err() {
                            info!("SharedPixelBuffer::new failed");
                            return;
                        }

                        let mut buf = buf.unwrap();
                        info!("new SharedPixelBuffer");
                        resp.read_exact(buf.make_mut_bytes()).unwrap();
                        info!("read finished");

                        u.upgrade_in_event_loop(|u| {
                            info!("set_photo_page_source");
                            u.set_photo_page_source(Image::from_rgb8(buf));
                        })
                        .unwrap();
                        thread::sleep(Duration::from_secs(5));
                    }
                });
            });
        }
    }

    fn bind_event_on_photo_page_request_next(&self) {
        info!("bind_event_on_photo_page_request_next");
        if let Some(ui) = self.app_window.as_weak().upgrade() {
            let c = self._http_client.clone();
            let u = ui.as_weak();
            ui.on_photo_page_request_next(move || {
                info!("on_photo_page_request_next");
                if let Some(ui) = u.upgrade() {
                    ui.set_photo_page_source(Default::default());
                }
                let c = c.clone();
                let u = u.clone();
                thread::spawn(move || {
                    let mut client = c.lock().unwrap();
                    let req = client
                        .request(Method::Get, "http://192.168.242.118:3000/api/photo", &[])
                        .unwrap();
                    let mut resp = req.submit().unwrap();
                    let mut byte_buf = [0u8; 2];
                    resp.read_exact(&mut byte_buf).unwrap();
                    let width = byte_buf[0] as u32;
                    let height = byte_buf[1] as u32;
                    info!("read frame: {}x{}", width, height);

                    // 这一步可能会内存分配失败
                    let buf = panic::catch_unwind(move || {
                        SharedPixelBuffer::<Rgb8Pixel>::new(width, height)
                    });
                    if buf.is_err() {
                        info!("SharedPixelBuffer::new failed");
                        return;
                    }
                    let mut buf = buf.unwrap();
                    info!("new SharedPixelBuffer");
                    resp.read_exact(buf.make_mut_bytes()).unwrap();
                    info!("read finished");

                    u.upgrade_in_event_loop(|u| {
                        info!("set_photo_page_source");
                        u.set_photo_page_source(Image::from_rgb8(buf));
                    })
                    .unwrap();
                });
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
}
