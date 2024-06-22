use std::rc::Rc;

use crate::{Context, Message, NodeName};

use crate::message::SystemMessage;

#[derive(Clone)]
pub struct SystemClient(pub Rc<dyn Context>);

impl SystemClient {
    pub fn get_free_heap_size(&self) -> usize {
        let r = self.0.sync_call(
            NodeName::System,
            Message::System(SystemMessage::GetFreeHeapSizeRequest),
        );
        match r.unwrap() {
            Message::System(SystemMessage::GetFreeHeapSizeResponse(s)) => s,
            m => panic!("unexpected response, {:?}", m),
        }
    }

    pub fn get_largeest_free_block(&self) -> usize {
        let r = self.0.sync_call(
            NodeName::System,
            Message::System(SystemMessage::GetLargestFreeBlock),
        );
        match r.unwrap() {
            Message::System(SystemMessage::GetLargestFreeBlockResponse(s)) => s,
            m => panic!("unexpected response, {:?}", m),
        }
    }

    pub fn get_fps(&self) -> usize {
        let r = self.0.sync_call(
            NodeName::System,
            Message::System(SystemMessage::GetFpsRequest),
        );
        match r.unwrap() {
            Message::System(SystemMessage::GetFpsResponse(s)) => s,
            m => panic!("unexpected response, {:?}", m),
        }
    }

    pub fn restart(&self) {
        self.0
            .sync_call(NodeName::System, Message::System(SystemMessage::Restart));
    }
}
