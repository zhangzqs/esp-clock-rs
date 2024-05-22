use app_core::{get_app_window, register_default_nodes, Scheduler};
use slint::ComponentHandle;
use std::time::Duration;

mod http;
mod timestamp;

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    let app = get_app_window();
    let mut sche = Scheduler::new();
    register_default_nodes(&mut sche);
    sche.register_node(http::HttpClient::new());
    sche.register_node(timestamp::TimestampClientService {});
    let sche_timer = slint::Timer::default();
    sche_timer.start(
        slint::TimerMode::Repeated,
        Duration::from_millis(10),
        move || {
            sche.schedule_once();
        },
    );
    if let Some(x) = app.upgrade() {
        x.run().unwrap();
    }
}
