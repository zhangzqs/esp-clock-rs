use std::rc::Rc;


use proto::{Context, HandleResult, Message, MessageTo, Node, NodeName};

pub struct HttpClientService {}

impl HttpClientService {
    pub fn new() -> Self {
        Self {}
    }
}

impl Node for HttpClientService {
    fn node_name(&self) -> NodeName {
        NodeName::HttpClient
    }

    fn handle_message(
        &mut self,
        ctx: Rc<dyn Context>,
        _from: NodeName,
        _to: MessageTo,
        msg: Message,
    ) -> HandleResult {
        {}
        HandleResult::Discard
    }
}
