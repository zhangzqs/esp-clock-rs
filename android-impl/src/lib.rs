use android_activity::{AndroidApp, InputStatus, MainEvent, PollEvent};
use slint_app::{MyApp, MyAppDeps, BootState};
use std::thread;
use std::time::Duration;
use desktop_svc::http::client::HttpClientAdapterConnection;
use std::rc::Rc;
use i_slint_backend_android_activity::AndroidPlatform;
#[no_mangle]
fn android_main(app: AndroidApp) {
    android_logger::init_once(android_logger::Config::default().with_min_level(log::Level::Info));

    let platform = AndroidPlatform::new(app);
    slint::platform::set_platform(Box::new(platform)).unwrap();
    
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
    
}