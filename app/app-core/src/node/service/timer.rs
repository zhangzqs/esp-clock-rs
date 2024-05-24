use std::rc::Rc;

use crate::proto::{
    Context, HandleResult, Message, MessageTo, MessageWithHeader, Node, NodeName, TimerMessage,
};

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

    fn handle_message(
        &self,
        ctx: Rc<dyn Context>,
        _from: NodeName,
        _to: MessageTo,
        msg: MessageWithHeader,
    ) -> HandleResult {
        if let Some(x) = msg.ready_result {
            return HandleResult::Finish(x);
        }

        if msg.is_pending {
            return HandleResult::Pending;
        }

        if let Message::Timer(TimerMessage::Request(x)) = msg.body {
            slint::Timer::single_shot(x, move || {
                ctx.async_ready(msg.seq, Message::Timer(TimerMessage::Response));
            });
            return HandleResult::Pending;
        }
        HandleResult::Discard
    }
}
