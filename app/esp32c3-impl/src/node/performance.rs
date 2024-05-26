use std::rc::Rc;

use app_core::proto::*;
use esp_idf_sys as _;

pub struct PerformanceService {}

impl PerformanceService {
    pub fn new() -> Self {
        Self {}
    }
}

impl Node for PerformanceService {
    fn node_name(&self) -> NodeName {
        NodeName::Performance
    }

    fn handle_message(&self, _ctx: Rc<dyn Context>, msg: MessageWithHeader) -> HandleResult {
        if let Message::Performance(pm) = msg.body {
            return HandleResult::Finish(Message::Performance(match pm {
                PerformanceMessage::GetFreeHeapSizeRequest => {
                    PerformanceMessage::GetFreeHeapSizeResponse(unsafe {
                        esp_idf_sys::esp_get_free_heap_size() as usize
                    })
                }
                PerformanceMessage::GetLargestFreeBlock => {
                    PerformanceMessage::GetLargestFreeBlockResponse(unsafe {
                        esp_idf_sys::heap_caps_get_largest_free_block(esp_idf_sys::MALLOC_CAP_8BIT)
                    })
                }
                PerformanceMessage::GetFpsRequest => PerformanceMessage::GetFpsResponse(60),
                m => panic!("unexpected message {m:?}"),
            }));
        }
        HandleResult::Discard
    }
}
