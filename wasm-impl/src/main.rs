use slint::ComponentHandle;
use std::{rc::Rc, time::Duration};

use log::debug;
use proto::{self, UtcDateTime};
use slint_app_v2::{get_app_window, get_schedular};
use wasm_logger;

struct DateTimeClientService {}
impl proto::Node for DateTimeClientService {
    fn node_name(&self) -> proto::NodeName {
        proto::NodeName::DateTimeClient
    }

    fn handle_message(
        &mut self,
        _ctx: Rc<dyn proto::Context>,
        _from: proto::NodeName,
        _to: proto::MessageTo,
        msg: proto::Message,
    ) -> proto::HandleResult {
        if let proto::Message::DateTime(proto::DateTimeMessage::UtcDateTimeRequest) = msg {
            let t = web_sys::js_sys::Date::new_0();
            return proto::HandleResult::Successful(proto::Message::DateTime(
                proto::DateTimeMessage::UtcDateTimeResponse(UtcDateTime {
                    year: t.get_utc_full_year() as _,
                    month: (t.get_utc_month() + 1) as _,
                    day: t.get_utc_date() as _,
                    hour: t.get_utc_hours() as _,
                    minute: t.get_utc_minutes() as _,
                    seconds: t.get_utc_seconds() as _,
                    week: t.get_utc_day() as _,
                }),
            ));
        }
        return proto::HandleResult::Discard;
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    let app = get_app_window();
    let mut sche = get_schedular();
    sche.register_node(DateTimeClientService {});
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
