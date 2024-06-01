use std::{rc::Rc, time::Duration};

use crate::proto::*;

pub struct MockWiFiService {}

impl MockWiFiService {
    pub fn new() -> Self {
        Self {}
    }
}

impl Node for MockWiFiService {
    fn node_name(&self) -> NodeName {
        NodeName::WiFi
    }

    fn handle_message(&self, ctx: Rc<dyn Context>, msg: MessageWithHeader) -> HandleResult {
        let seq = msg.seq;
        if let Message::WiFi(msg) = msg.body {
            match msg {
                WiFiMessage::ConnectRequest(_) => {
                    slint::Timer::single_shot(Duration::from_secs(9), move || {
                        ctx.async_ready(seq, Message::WiFi(WiFiMessage::ConnectResponse));
                    });
                    return HandleResult::Pending;
                }
                m => panic!("unexpected request message: {m:?}"),
            }
        }
        HandleResult::Discard
    }
}
