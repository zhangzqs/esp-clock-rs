use std::rc::Rc;

use crate::proto::*;
pub struct MockPerformanceService {}

impl Node for MockPerformanceService {
    fn node_name(&self) -> NodeName {
        NodeName::Performance
    }

    fn handle_message(
        &self,
        _ctx: Rc<dyn Context>,
        _from: NodeName,
        _to: MessageTo,
        msg: MessageWithHeader,
    ) -> HandleResult {
        if let Message::Performance(pm) = msg.body {
            return HandleResult::Finish(Message::Performance(match pm {
                PerformanceMessage::GetFreeHeapSizeRequest => {
                    PerformanceMessage::GetFreeHeapSizeResponse(666)
                }
                PerformanceMessage::GetLargestFreeBlock => {
                    PerformanceMessage::GetLargestFreeBlockResponse(999)
                }
                PerformanceMessage::GetFpsRequest => PerformanceMessage::GetFpsResponse(60),
                m => panic!("unexpected message {m:?}"),
            }));
        }
        HandleResult::Discard
    }
}
