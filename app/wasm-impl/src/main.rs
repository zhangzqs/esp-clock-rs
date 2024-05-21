use log::{debug, info};
use proto::{HandleResult, HttpMessage, HttpRequestMethod, HttpResponse, Message};
use slint::ComponentHandle;
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
    sync::Arc,
    time::Duration,
};
use time::OffsetDateTime;

use app_core::{get_app_window, get_scheduler, register_default_nodes, Scheduler};

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

impl app_core::Platform for WasmPlatform {
    fn duration_since_init(&self) -> Duration {
        let a = OffsetDateTime::now_utc() - self.start;
        Duration::from_nanos(a.whole_nanoseconds() as u64)
    }
}

fn convert(method: HttpRequestMethod) -> reqwest::Method {
    use reqwest::Method;
    match method {
        HttpRequestMethod::Get => Method::GET,
    }
}

struct HttpClient {
    // 还在执行中的消息
    running_req: Rc<RefCell<HashSet<u32>>>,
    // 已经就绪的响应
    ready_resp: Rc<RefCell<HashMap<u32, Arc<HttpResponse>>>>,
}

impl HttpClient {
    fn new() -> Self {
        Self {
            running_req: Rc::new(RefCell::new(HashSet::new())),
            ready_resp: Rc::new(RefCell::new(HashMap::new())),
        }
    }
}

impl proto::Node for HttpClient {
    fn node_name(&self) -> proto::NodeName {
        proto::NodeName::HttpClient
    }

    fn handle_message(
        &mut self,
        _ctx: Rc<dyn proto::Context>,
        _from: proto::NodeName,
        _to: proto::MessageTo,
        msg: proto::MessageWithHeader,
    ) -> proto::HandleResult {
        match msg.body {
            Message::Http(HttpMessage::Request(req)) => {
                if self.running_req.borrow().contains(&msg.seq) {
                    // 若消息仍处于running态，继续返回 Pending，调度器后续继续轮询
                    return HandleResult::Pending;
                } else if self.ready_resp.borrow().contains_key(&msg.seq) {
                    // 若消息结果为ready态，则返回Sucessful
                    return HandleResult::Successful(Message::Http(HttpMessage::Response(
                        self.ready_resp.borrow_mut().remove(&msg.seq).unwrap(),
                    )));
                } else {
                    // 否则为新消息
                    self.running_req.borrow_mut().insert(msg.seq);
                    let req = req.clone();
                    let running_req = self.running_req.clone();
                    let ready_resp = self.ready_resp.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        let resp = reqwest::Client::new()
                            .execute(
                                reqwest::Client::new()
                                    .request(convert(req.method.clone()), req.url.clone())
                                    .build()
                                    .unwrap(),
                            )
                            .await
                            .unwrap();
                        let body = resp.bytes().await.unwrap().to_vec();
                        running_req.borrow_mut().remove(&msg.seq);
                        ready_resp.borrow_mut().insert(
                            msg.seq,
                            Arc::new(HttpResponse {
                                request: req,
                                body: proto::HttpBody::Bytes(body),
                            }),
                        );
                    });
                    return HandleResult::Pending;
                }
            }
            _ => {}
        }
        return proto::HandleResult::Discard;
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    let app = get_app_window();
    let mut sche = Scheduler::new_with_platform(WasmPlatform::new());
    register_default_nodes(&mut sche);
    sche.register_node(HttpClient::new());
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
