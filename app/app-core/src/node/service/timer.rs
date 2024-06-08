use std::{rc::Rc, time::Duration};

use crate::proto::*;

pub struct TimerService {}

impl TimerService {
    pub fn new() -> Self {
        Self {}
    }
}

impl Node for TimerService {
    fn node_name(&self) -> NodeName {
        NodeName::Timer
    }

    fn handle_message(&self, ctx: Rc<dyn Context>, msg: MessageWithHeader) -> HandleResult {
        if let Message::Timer(TimerMessage::Request(x)) = msg.body {
            slint::Timer::single_shot(Duration::from_millis(x as _), move || {
                ctx.async_ready(msg.seq, Message::Timer(TimerMessage::Response));
            });
            return HandleResult::Pending;
        }
        HandleResult::Discard
    }
}
