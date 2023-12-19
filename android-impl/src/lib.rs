use android_activity::{AndroidApp, InputStatus, MainEvent, PollEvent};
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics_group::DisplayGroup;
use embedded_svc::http::{server::Handler, Method};
use embedded_tone::RawTonePlayer;
use log::info;
use slint_app::{BootState, EvilApple, LEDController, MockSystem, MyApp, MyAppDeps};
use std::{
    sync::{Arc, Mutex},
    thread,
};

use desktop_svc::http::{
    client::HttpClientConnection,
    server::{Configuration, HttpServer},
};
use i_slint_backend_android_activity::AndroidPlatform;
use std::rc::Rc;
use std::time::Duration;

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

#[no_mangle]
fn android_main(app: AndroidApp) {
    android_logger::init_once(android_logger::Config::default().with_min_level(log::Level::Info));

    slint::platform::set_platform(Box::new(AndroidPlatform::new(app))).unwrap();
    info!("Android Main");

    let physical_display = Arc::new(Mutex::new(embedded_graphics::mock_display::MockDisplay::<
        Rgb888,
    >::new()));
    let display_group = Arc::new(Mutex::new(DisplayGroup::new(physical_display.clone(), 2)));

    let app = Rc::new(MyApp::new(MyAppDeps {
        http_conn: HttpClientConnection::new(),
        system: MockSystem,
        display_group,
        player: MockPlayer,
        eval_apple: MockEvilApple,
        screen_brightness_controller: MockLEDController::default(),
        blue_led: MockLEDController::default(),
        http_server: std::marker::PhantomData::<HttpServerWrapper>,
    }));

    let u = app.get_app_window();
    thread::spawn(move || {
        u.upgrade_in_event_loop(|ui| {
            ui.invoke_set_boot_state(BootState::Booting);
        }).unwrap();
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

    app.run().unwrap();
}
