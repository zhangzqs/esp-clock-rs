use std::rc::Rc;

use time::OffsetDateTime;

use crate::proto::*;

pub struct DefaultTimestampService {}

impl Node for DefaultTimestampService {
    fn node_name(&self) -> NodeName {
        NodeName::TimestampClient
    }

    fn handle_message(&self, _ctx: Rc<dyn Context>, msg: MessageWithHeader) -> HandleResult {
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
