use app_core::{get_app_window, get_scheduler};
use slint::ComponentHandle;
use std::time::Duration;

mod console;
mod http;
mod midiplayer;
mod storage;

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    let app = get_app_window();
    let sche = get_scheduler();
    sche.register_node(http::HttpClient::new());
    sche.register_node(storage::LocalStorageService::new());
    sche.register_node(midiplayer::MidiPlayerService::new());
    sche.register_node(console::ConsoleNode::new());
    let sche_timer = slint::Timer::default();
    sche_timer.start(
        slint::TimerMode::Repeated,
        Duration::from_millis(16),
        move || {
            sche.schedule_once();
        },
    );
    if let Some(x) = app.upgrade() {
        x.run().unwrap();
    }
}
