use slint::ComponentHandle;
use std::{rc::Rc, time::Duration};
use time::OffsetDateTime;

use app_core::{get_app_window, get_scheduler, register_default_nodes, Platform, Scheduler};

struct TimestampClientService {}
impl proto::Node for TimestampClientService {
    fn node_name(&self) -> proto::NodeName {
        proto::NodeName::TimestampClient
    }

    fn handle_message(
        &mut self,
        _ctx: Rc<dyn proto::Context>,
        _from: proto::NodeName,
        _to: proto::MessageTo,
        msg: proto::MessageWithHeader,
    ) -> proto::HandleResult {
        if let proto::Message::DateTime(proto::TimeMessage::GetTimestampNanosRequest) = msg.body {
            let t = web_sys::js_sys::Date::now();
            return proto::HandleResult::Successful(proto::Message::DateTime(
                proto::TimeMessage::GetTimestampNanosResponse(t as i128 * 1_000_000),
            ));
        }
        proto::HandleResult::Discard
    }
}

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

impl Platform for WasmPlatform {
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
    sche.register_node(TimestampClientService {});
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
