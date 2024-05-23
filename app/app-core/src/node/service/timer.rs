use std::{cell::RefCell, collections::HashSet, rc::Rc};

use crate::proto::{
    Context, HandleResult, Message, MessageTo, MessageWithHeader, Node, NodeName, TimeMessage,
    TimerMessage,
};

pub struct TimerService {
    ready_resp: Rc<RefCell<HashSet<u32>>>,
}

impl TimerService {
    pub fn new() -> Self {
        Self {
            ready_resp: Rc::new(RefCell::new(HashSet::new())),
        }
    }
}

impl Node for TimerService {
    fn node_name(&self) -> NodeName {
        NodeName::Timer
    }

    fn handle_message(
        &mut self,
        _ctx: Rc<dyn Context>,
        _from: NodeName,
        _to: MessageTo,
        msg: MessageWithHeader,
    ) -> HandleResult {
        if self.ready_resp.borrow().contains(&msg.seq) {
            self.ready_resp.borrow_mut().remove(&msg.seq);
            return HandleResult::Finish(Message::Timer(TimerMessage::Response));
        }

        if msg.is_pending {
            return HandleResult::Pending;
        }

        if let Message::Timer(TimerMessage::Request(x)) = msg.body {
            let ready_resp = self.ready_resp.clone();
            slint::Timer::single_shot(x, move || {
                ready_resp.borrow_mut().insert(msg.seq);
            });
            return HandleResult::Pending;
        }
        HandleResult::Discard
    }
}
