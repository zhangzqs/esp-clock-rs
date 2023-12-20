use android_activity::{AndroidApp, InputStatus, MainEvent, PollEvent};
use button_driver::{Button, ButtonConfig, PinWrapper};
use embedded_graphics::{
    geometry::{Point, Size},
    pixelcolor::Rgb888,
    primitives::Rectangle,
};
use embedded_graphics_group::{DisplayGroup, LogicalDisplay};
use embedded_svc::http::{server::Handler, Method};
use embedded_tone::RawTonePlayer;
use log::{debug, info};
use slint::{Image, SharedPixelBuffer};
use slint_app::{BootState, EvilApple, LEDController, MockSystem, MyApp, MyAppDeps};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
};

use desktop_svc::http::{
    client::HttpClientConnection,
    server::{Configuration, HttpServer},
};
use i_slint_backend_android_activity::AndroidPlatform;
use std::rc::Rc;
use std::time::Duration;

pub struct RodioPlayer {
    stream: rodio::OutputStream,
    stream_handle: rodio::OutputStreamHandle,
    sink: rodio::Sink,
}

unsafe impl Send for RodioPlayer {}

impl RodioPlayer {
    pub fn new() -> Self {
        let (stream, stream_handle) = rodio::OutputStream::try_default().unwrap();

        let sink = rodio::Sink::try_new(&stream_handle).unwrap();

        Self {
            sink,
            stream,
            stream_handle,
        }
    }
}

impl RawTonePlayer for RodioPlayer {
    fn tone(&mut self, freq: u32) {
        debug!("tone {}", freq);
        self.sink.stop();
        let now = std::time::Instant::now();
        self.sink.append(rodio::source::SineWave::new(freq as f32));
        debug!("append takes {:?}", now.elapsed());
    }

    fn off(&mut self) {
        self.sink.stop();
    }
}

struct MockEvilApple;

impl EvilApple for MockEvilApple {
    fn attack_once(&self, _data: &[u8]) {
        info!("attack once");
    }
}

struct MockLEDController {
    brightness: u32,
}

impl Default for MockLEDController {
    fn default() -> Self {
        Self { brightness: 1000 }
    }
}

impl LEDController for MockLEDController {
    fn get_max_brightness(&self) -> u32 {
        info!("get max brightness");
        1000
    }

    fn set_brightness(&mut self, brightness: u32) {
        info!("set brightness {}", brightness);
        self.brightness = brightness;
    }

    fn get_brightness(&self) -> u32 {
        info!("get brightness");
        self.brightness
    }
}

struct MockPlayer;

impl RawTonePlayer for MockPlayer {
    fn tone(&mut self, freq: u32) {
        info!("tone {}", freq);
    }

    fn off(&mut self) {
        info!("off");
    }
}

struct HttpServerWrapper<'a>(desktop_svc::http::server::HttpServer<'a>);

impl<'a: 'static> slint_app::Server<'a> for HttpServerWrapper<'a> {
    type Conn<'r> = desktop_svc::http::server::HttpServerConnection;
    type HttpServerError = desktop_svc::http::server::HttpServerError;

    fn new() -> Self {
        let server = HttpServer::new(&Configuration {
            http_port: 8080,
            uri_match_wildcard: true,
        })
        .unwrap();
        HttpServerWrapper(server)
    }
    fn handler<H>(
        &mut self,
        uri: &str,
        method: Method,
        handler: H,
    ) -> Result<&mut Self, Self::HttpServerError>
    where
        H: for<'r> Handler<Self::Conn<'r>> + Send + 'a,
    {
        self.0.handler(uri, method, handler)?;
        Ok(self)
    }
}

#[derive(Clone)]
struct MyButtonPin(Rc<AtomicBool>);

impl PinWrapper for MyButtonPin {
    fn is_high(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}

#[no_mangle]
fn android_main(app: AndroidApp) {
    android_logger::init_once(android_logger::Config::default().with_min_level(log::Level::Info));

    slint::platform::set_platform(Box::new(AndroidPlatform::new(app))).unwrap();
    info!("Android Main");

    let buf = Arc::new(Mutex::new(SharedPixelBuffer::<slint::Rgb8Pixel>::new(
        240, 240,
    )));
    let physical_display = Arc::new(Mutex::new(
        embedded_graphics_slint_image_buf::SlintPixelBufferDrawTarget { buf: buf.clone() },
    ));
    let display_group = Arc::new(Mutex::new(DisplayGroup::new(physical_display.clone(), 2)));
    let mock_main_logical_display = LogicalDisplay::new(
        display_group.clone(),
        Rectangle::new(Point::new(0, 0), Size::new(240, 240)),
    );
    let mock_main_logical_display_id = mock_main_logical_display.lock().unwrap().get_id() as isize;
    display_group
        .lock()
        .unwrap()
        .switch_to_logical_display(mock_main_logical_display_id);

    let app = Rc::new(MyApp::new(MyAppDeps {
        http_conn: HttpClientConnection::new(),
        system: MockSystem,
        display_group: display_group.clone(),
        player: RodioPlayer::new(),
        eval_apple: MockEvilApple,
        screen_brightness_controller: MockLEDController::default(),
        blue_led: MockLEDController::default(),
        http_server: std::marker::PhantomData::<HttpServerWrapper>,
    }));

    let u = app.get_app_window();
    let display_group_timer = slint::Timer::default();
    display_group_timer.start(
        slint::TimerMode::Repeated,
        Duration::from_millis(100),
        move || {
            let mut display_group = display_group.lock().unwrap();
            let show_main = display_group.get_current_active_display_index() == 0;
            drop(display_group);
            if let Some(ui) = u.upgrade() {
                ui.set_show_external_display(!show_main);
                if !show_main {
                    ui.set_external_display_image(Image::from_rgb8(buf.lock().unwrap().clone()));
                }
            }
        },
    );

    let u = app.get_app_window();
    thread::spawn(move || {
        u.upgrade_in_event_loop(|ui| {
            ui.invoke_set_boot_state(BootState::Booting);
        })
        .unwrap();
        thread::sleep(Duration::from_secs(1));
        u.upgrade_in_event_loop(|ui| {
            ui.invoke_set_boot_state(BootState::Connecting);
        })
        .unwrap();
        thread::sleep(Duration::from_secs(5));
        u.upgrade_in_event_loop(|ui| {
            ui.invoke_set_boot_state(BootState::BootSuccess);
        })
        .unwrap();
        thread::sleep(Duration::from_secs(1));
        u.upgrade_in_event_loop(|ui| {
            ui.invoke_set_boot_state(BootState::Finished);
        })
        .unwrap();
    });

    // 分发按键事件
    // 假设代表按键状态，默认为松开，值为false
    let button_state = Rc::new(AtomicBool::new(false));
    let mut button = Button::new(
        MyButtonPin(button_state.clone()),
        ButtonConfig {
            mode: button_driver::Mode::PullDown, // 当按键松开时，是低电平
            ..Default::default()
        },
    );

    let button_event_timer = slint::Timer::default();
    let u = app.get_app_window();
    button_event_timer.start(
        slint::TimerMode::Repeated,
        Duration::from_millis(10),
        move || {
            button.tick();
            if button.clicks() > 0 {
                let clicks = button.clicks();
                debug!("Clicks: {}", clicks);
                if let Some(ui) = u.upgrade() {
                    ui.invoke_on_one_button_clicks(clicks as i32);
                }
            } else if let Some(dur) = button.current_holding_time() {
                debug!("Held for {dur:?}");
                if let Some(ui) = u.upgrade() {
                    ui.invoke_on_one_button_long_pressed_holding(dur.as_millis() as i64);
                }
            } else if let Some(dur) = button.held_time() {
                debug!("Total holding time {dur:?}");
                if let Some(ui) = u.upgrade() {
                    ui.invoke_on_one_button_long_pressed_held(dur.as_millis() as i64);
                }
            }
            button.reset();
        },
    );

    let button_state = button_state.clone();
    if let Some(ui) = app.get_app_window().upgrade() {
        ui.on_touch_area_pointer_event(move |e| {
            let kind = format!("{}", e.kind);
            match kind.as_str() {
                "down" => {
                    button_state.store(true, Ordering::Relaxed);
                }
                "up" => {
                    button_state.store(false, Ordering::Relaxed);
                }
                _ => {}
            }
        });
    }

    app.run().unwrap();
}
