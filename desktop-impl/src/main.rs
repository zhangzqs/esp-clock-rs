use std::{cell::RefCell, rc::Rc, time::Duration, env::set_var, thread};

use desktop_svc::http::client::HttpClientAdapterConnection;

use log::info;

use slint_app::{MyApp, MyAppDeps, BootState};

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
    info!("Starting desktop simulator");

    let app = Rc::new(MyApp::new(MyAppDeps {
        http_conn: HttpClientAdapterConnection::new(),
    }));

    app.set_boot_state(BootState::Booting);
    let u = app.get_app_window_as_weak();
    thread::spawn(move ||{
        thread::sleep(Duration::from_secs(1));
        u.upgrade_in_event_loop(|ui| {
            ui.invoke_set_boot_state(BootState::Connecting);
        }).unwrap();
        thread::sleep(Duration::from_secs(5));
        u.upgrade_in_event_loop(|ui| {
            ui.invoke_set_boot_state(BootState::BootSuccess);
        }).unwrap();
        thread::sleep(Duration::from_secs(1));
        u.upgrade_in_event_loop(|ui| {
            ui.invoke_set_boot_state(BootState::Finished);
        }).unwrap();
    });
    slint::platform::
    app.run().unwrap();
}
