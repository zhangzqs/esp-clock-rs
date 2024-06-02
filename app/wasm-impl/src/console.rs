use std::sync::{Arc, Mutex, OnceLock};

use app_core::proto::*;
use log::info;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConsoleMessage {
    to: MessageTo,
    body: Message,
}

struct QueueItem {
    message: ConsoleMessage,
    callback: Box<dyn FnOnce(HandleResult) + 'static>,
}

unsafe impl Send for QueueItem {}

static QUEUE: OnceLock<Arc<Mutex<Vec<QueueItem>>>> = OnceLock::new();

fn get_queue() -> Arc<Mutex<Vec<QueueItem>>> {
    QUEUE
        .get_or_init(|| Arc::new(Mutex::new(Vec::new())))
        .clone()
}

fn push_queue_item(message: ConsoleMessage, callback: Box<dyn FnOnce(HandleResult)>) {
    get_queue()
        .lock()
        .unwrap()
        .push(QueueItem { message, callback });
}

#[wasm_bindgen::prelude::wasm_bindgen]
pub fn send_message(message: String, callback: web_sys::js_sys::Function) {
    push_queue_item(
        serde_json::from_str(&message).unwrap(),
        Box::new(move |r| {
            info!("js queue ret: {r:?}");
            let s = serde_json::to_string(&r).unwrap();
            let js_s = JsValue::from_str(&s);
            let this = JsValue::null();
            callback.call1(&this, &js_s).unwrap();
        }),
    );
}

fn pop_queue_item() -> Option<QueueItem> {
    get_queue().lock().unwrap().pop()
}

pub struct ConsoleNode {}

impl ConsoleNode {
    pub fn new() -> Self {
        Self {}
    }
}

impl Node for ConsoleNode {
    fn node_name(&self) -> NodeName {
        NodeName::Other("WebConsole".into())
    }

    fn handle_message(
        &self,
        ctx: std::rc::Rc<dyn Context>,
        _msg: MessageWithHeader,
    ) -> HandleResult {
        if let Some(QueueItem {
            message: ConsoleMessage { to, body },
            callback,
        }) = pop_queue_item()
        {
            match to {
                MessageTo::Broadcast => {
                    ctx.broadcast_global(body);
                }
                MessageTo::Topic(topic) => {
                    ctx.broadcast_topic(topic, body);
                }
                MessageTo::Point(node) => {
                    ctx.async_call(node, body, callback);
                }
            }
        }
        HandleResult::Discard
    }
}
