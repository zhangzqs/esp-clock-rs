use slint::ComponentHandle;
use std::time::Duration;
use time::OffsetDateTime;

use app_core::{get_app_window, register_default_nodes, Scheduler};

mod http;
mod timestamp;

struct WasmPlatform {
    start: OffsetDateTime,
}

impl WasmPlatform {
    fn new() -> Self {
        Self {
            start: OffsetDateTime::now_utc(),
        }
    }
}

impl app_core::Platform for WasmPlatform {
    fn duration_since_init(&self) -> Duration {
        let a = OffsetDateTime::now_utc() - self.start;
        Duration::from_nanos(a.whole_nanoseconds() as u64)
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    let app = get_app_window();
    let mut sche = Scheduler::new_with_platform(WasmPlatform::new());
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
