use std::{cell::RefCell, rc::Rc, time::Duration, env::set_var, thread, sync::{Arc, Mutex}};

use desktop_svc::http::client::HttpClientAdapterConnection;

use embedded_graphics::{pixelcolor::{Rgb888, raw::BigEndian}, framebuffer::Framebuffer};
use embedded_graphics_group::DisplayGroup;
use embedded_tone::MockPlayer;
use log::info;

use slint_app::{MyApp, MyAppDeps, BootState, MockSystem};

use button_driver::PinWrapper;

#[derive(Clone)]
struct MyButtonPin(Rc<RefCell<bool>>);

impl PinWrapper for MyButtonPin {
    fn is_high(&self) -> bool {
        *self.0.borrow()
    }
}

fn main() {
    set_var("RUST_LOG", "debug");
    env_logger::init();
    info!("Starting desktop implementation...");

    let mut physical_display = Arc::new(Mutex::new(Framebuffer::<Rgb888, _, BigEndian, 240, 240, 172800>::new()));

    let display_group = Arc::new(Mutex::new(DisplayGroup::new(physical_display.clone(), 2)));

    let app = MyApp::new(MyAppDeps {
        http_conn: HttpClientAdapterConnection::new(),
        system: MockSystem,
        display_group: display_group.clone(),
        player: MockPlayer::default(),
    });


    // 模拟启动过程
    let u = app.get_app_window();
    if let Some(ui) = u.upgrade() { ui.invoke_set_boot_state(BootState::Booting); }
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(1));
        u.upgrade_in_event_loop(|ui| {
            ui.invoke_set_boot_state(BootState::Connecting);
        })
        .unwrap();
        thread::sleep(Duration::from_secs(1));
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
