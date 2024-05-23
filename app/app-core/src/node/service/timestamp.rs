use std::rc::Rc;

use time::OffsetDateTime;

use crate::proto::{
    Context, HandleResult, Message, MessageTo, MessageWithHeader, Node, NodeName, TimeMessage,
};

pub struct DefaultTimestampService {}

impl Node for DefaultTimestampService {
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
            return HandleResult::Finish(Message::DateTime(
                TimeMessage::GetTimestampNanosResponse(
                    OffsetDateTime::now_utc().unix_timestamp_nanos(),
                ),
            ));
        }
        HandleResult::Discard
    }
}
