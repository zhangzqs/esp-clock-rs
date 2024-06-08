use std::rc::Rc;

use crate::proto::*;
pub struct MockSystemService {}

impl Node for MockSystemService {
    fn node_name(&self) -> NodeName {
        NodeName::System
    }

    fn handle_message(&self, _ctx: Rc<dyn Context>, msg: MessageWithHeader) -> HandleResult {
        if let Message::System(pm) = msg.body {
            return HandleResult::Finish(Message::System(match pm {
                SystemMessage::GetFreeHeapSizeRequest => {
                    SystemMessage::GetFreeHeapSizeResponse(666)
                }
                SystemMessage::GetLargestFreeBlock => {
                    SystemMessage::GetLargestFreeBlockResponse(999)
                }
                SystemMessage::GetFpsRequest => SystemMessage::GetFpsResponse(60),
                m => panic!("unexpected message {m:?}"),
            }));
        }
        HandleResult::Discard
    }
}
