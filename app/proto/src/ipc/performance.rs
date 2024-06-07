use std::rc::Rc;

use crate::{Context, Message, NodeName};

use crate::message::PerformanceMessage;

#[derive(Clone)]
pub struct PerformanceClient(pub Rc<dyn Context>);

impl PerformanceClient {
    pub fn get_free_heap_size(&self) -> usize {
        let r = self.0.sync_call(
            NodeName::Performance,
            Message::Performance(PerformanceMessage::GetFreeHeapSizeRequest),
        );
        match r.unwrap() {
            Message::Performance(PerformanceMessage::GetFreeHeapSizeResponse(s)) => s,
            m => panic!("unexpected response, {:?}", m),
        }
    }

    pub fn get_largeest_free_block(&self) -> usize {
        let r = self.0.sync_call(
            NodeName::Performance,
            Message::Performance(PerformanceMessage::GetLargestFreeBlock),
        );
        match r.unwrap() {
            Message::Performance(PerformanceMessage::GetLargestFreeBlockResponse(s)) => s,
            m => panic!("unexpected response, {:?}", m),
        }
    }

    pub fn get_fps(&self) -> usize {
        let r = self.0.sync_call(
            NodeName::Performance,
            Message::Performance(PerformanceMessage::GetFpsRequest),
        );
        match r.unwrap() {
            Message::Performance(PerformanceMessage::GetFpsResponse(s)) => s,
            m => panic!("unexpected response, {:?}", m),
        }
    }
}
