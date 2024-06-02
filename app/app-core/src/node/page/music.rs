use crate::proto::*;
use std::rc::Rc;

pub struct MusicPage {}

impl MusicPage {
    pub fn new() -> Self {
        Self {}
    }
}

impl Node for MusicPage {
    fn node_name(&self) -> NodeName {
        NodeName::MusicPage
    }

    fn handle_message(&self, _ctx: Rc<dyn Context>, msg: MessageWithHeader) -> HandleResult {
        match msg.body {
            Message::Lifecycle(msg) => {}
            Message::OneButton(msg) => {}
            _ => {}
        }
        HandleResult::Discard
    }
}
