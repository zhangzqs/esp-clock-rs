use std::rc::Rc;

use app_core::proto::{
    Context, HandleResult, Message, MessageTo, MessageWithHeader, Node, NodeName, TimeMessage,
};

pub struct TimestampClientService {}
impl Node for TimestampClientService {
    fn node_name(&self) -> NodeName {
        NodeName::TimestampClient
    }

    fn handle_message(
        &mut self,
        _ctx: Rc<dyn Context>,
        _from: NodeName,
        _to: MessageTo,
        msg: MessageWithHeader,
    ) -> HandleResult {
        if let Message::DateTime(TimeMessage::GetTimestampNanosRequest) = msg.body {
            let t = web_sys::js_sys::Date::now();
            return HandleResult::Successful(Message::DateTime(
                TimeMessage::GetTimestampNanosResponse(t as i128 * 1_000_000),
            ));
        }
        HandleResult::Discard
    }
}
