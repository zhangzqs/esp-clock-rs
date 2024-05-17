use slint::{ComponentHandle, Weak};

use crate::proto::{Node, NodeName, Context, HandleResult, LifecycleMessage, Message, MessageTo};
use crate::ui::{AppWindow, PageRouteTable, PageRouter};

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
        ctx: Box<dyn Context>,
        _from: NodeName,
        _to: MessageTo,
        msg: Message,
    ) -> HandleResult {
        match msg {
            
            _ => {}
        }
        HandleResult::Discard
    }
}
